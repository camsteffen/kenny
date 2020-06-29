//! Generate images for unsolved or solved puzzles

use crate::collections::square::{Coord, IsSquare, Square};
use crate::puzzle::solve::markup::{CellChange, CellChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::{Puzzle, Solution, Value};
use ahash::AHashSet;
use image::{Pixel, RgbImage};
use itertools::Itertools;
use once_cell::sync::Lazy;
use rusttype::point;
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::PositionedGlyph;
use rusttype::Scale;
use std::cmp::{max, min};
use std::ops::Deref;

type Rgb = image::Rgb<u8>;

const BLACK: Rgb = image::Rgb([0; 3]);
const WHITE: Rgb = image::Rgb([255; 3]);
const RED: Rgb = image::Rgb([255, 0, 0]);

const COLOR_CELL_BORDER: Rgb = image::Rgb([205; 3]);
const COLOR_CAGE_BORDER: Rgb = BLACK;
const COLOR_BG: Rgb = WHITE;
const COLOR_HIGHLIGHT_BG: Rgb = image::Rgb([255, 255, 200]);

const CELL_BORDER_RATIO: u32 = 25;
const DEFAULT_CELL_WIDTH: u32 = 60;

static FONT: Lazy<Font<'static>> = Lazy::new(|| {
    let bytes: &[u8] = include_bytes!("../../res/Roboto-Regular.ttf");
    let font_collection = FontCollection::from_bytes(bytes).expect("Error loading font collection");
    font_collection.font_at(0).expect("load font")
});

// todo svg format? compose with svg and render with resvg
/// Creates an image of a puzzle with optional markup
pub struct PuzzleImageBuilder<'a> {
    puzzle: &'a Puzzle,

    cell_variables: Option<&'a Square<CellVariable>>,
    solution: Option<&'a Solution>,
    cell_changes: Option<&'a CellChanges>,

    image_width: u32,
    cell_width: u32,
    border_width: u32,
}

impl<'a> PuzzleImageBuilder<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        let cell_width = DEFAULT_CELL_WIDTH;
        let border_width = Self::calc_border_width(cell_width);
        let image_width = Self::calc_image_width(cell_width, puzzle.width(), border_width);

        Self {
            puzzle,
            image_width,
            border_width,
            cell_width,
            cell_variables: None,
            solution: None,
            cell_changes: None,
        }
    }

    /// Sets the width of a cell in pixels
    pub fn cell_width(&mut self, cell_width: u32) -> &mut Self {
        self.cell_width = cell_width;
        self.border_width = Self::calc_border_width(cell_width);
        self.image_width =
            Self::calc_image_width(cell_width, self.puzzle.width(), self.border_width);
        self
    }

    /// Sets the width of the image (approximate)
    pub fn image_width(&mut self, image_width: u32) -> &mut Self {
        let a = CELL_BORDER_RATIO
            .checked_mul(image_width)
            .expect("image width is too big");
        let b = CELL_BORDER_RATIO * self.puzzle.width() as u32 + 1;
        self.cell_width(a / b)
    }

    pub(crate) fn cell_variables(
        &mut self,
        cell_variables: Option<&'a Square<CellVariable>>,
    ) -> &mut Self {
        self.cell_variables = cell_variables;
        self
    }

    pub(crate) fn cell_changes(&mut self, cell_changes: &'a CellChanges) -> &mut Self {
        self.cell_changes = Some(cell_changes);
        self
    }

    pub fn solution(&mut self, solution: &'a Solution) -> &mut Self {
        self.solution = Some(solution);
        self
    }

    pub fn build(self) -> RgbImage {
        let buffer = RgbImage::from_pixel(self.image_width, self.image_width, COLOR_BG);
        let context = BuildContext {
            builder: self,
            buffer,
            font: &FONT,
        };
        context.build()
    }

    fn calc_border_width(cell_width: u32) -> u32 {
        cell_width / CELL_BORDER_RATIO
    }

    fn calc_image_width(cell_width: u32, puzzle_width: usize, border_width: u32) -> u32 {
        cell_width * puzzle_width as u32 + border_width
    }
}

impl<'a> Deref for BuildContext<'a> {
    type Target = PuzzleImageBuilder<'a>;

    fn deref(&self) -> &Self::Target {
        &self.builder
    }
}

struct BuildContext<'a> {
    builder: PuzzleImageBuilder<'a>,
    buffer: RgbImage,
    font: &'static Font<'static>,
}

impl BuildContext<'_> {
    fn build(mut self) -> RgbImage {
        self.draw_grid();
        self.highlight_cells();
        self.draw_cage_glyphs();
        if let Some(cell_variables) = self.cell_variables {
            self.draw_markup(cell_variables);
        } else if let Some(solution) = self.solution {
            self.draw_solution(solution);
        }
        self.buffer
    }

    fn draw_grid(&mut self) {
        // let image_width = self.cell_width * self.puzzle.width() as u32 + self.border_width;
        let cells_width = self.cell_width * self.puzzle.width() as u32;

        // draw outer border
        self.draw_rectangle(0, 0, cells_width, self.border_width, COLOR_CAGE_BORDER);
        self.draw_rectangle(
            cells_width,
            0,
            self.image_width,
            cells_width,
            COLOR_CAGE_BORDER,
        );
        self.draw_rectangle(
            self.border_width,
            cells_width,
            self.image_width,
            self.image_width,
            COLOR_CAGE_BORDER,
        );
        self.draw_rectangle(
            0,
            self.border_width,
            self.border_width,
            self.image_width,
            COLOR_CAGE_BORDER,
        );

        // draw horizontal line segments
        for i in 1..self.puzzle.width() {
            // row
            for j in 0..self.puzzle.width() {
                // col
                let pos1 = Coord::new(j, i - 1);
                let pos2 = Coord::new(j, i);
                let color = if self.puzzle.cell(pos1).cage_id() == self.puzzle.cell(pos2).cage_id()
                {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width + self.border_width;
                let y1 = i as u32 * self.cell_width;
                let x2 = x1 + self.cell_width - self.border_width;
                let y2 = y1 + self.border_width;
                self.draw_rectangle(x1, y1, x2, y2, color);
            }
        }

        // draw vertical line segments
        for i in 0..self.puzzle.width() {
            // row
            for j in 1..self.puzzle.width() {
                // col
                let pos1 = Coord::new(j - 1, i);
                let pos2 = Coord::new(j, i);
                let color = if self.puzzle.cell(pos1).cage_id() == self.puzzle.cell(pos2).cage_id()
                {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width;
                let y1 = i as u32 * self.cell_width + self.border_width;
                let x2 = x1 + self.border_width;
                let y2 = y1 + self.cell_width - self.border_width;
                self.draw_rectangle(x1, y1, x2, y2, color);
            }
        }

        // draw intersections
        for i in 1..self.puzzle.width() {
            for j in 1..self.puzzle.width() {
                let pos = [
                    Coord::new(j - 1, i - 1),
                    Coord::new(j - 1, i),
                    Coord::new(j, i - 1),
                    Coord::new(j, i),
                ];
                let same_cage = pos
                    .iter()
                    .map(|pos| self.puzzle.cell(*pos).cage_id())
                    .all_equal();
                let color = if same_cage {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width;
                let y1 = i as u32 * self.cell_width;
                let x2 = x1 + self.border_width;
                let y2 = y1 + self.border_width;
                self.draw_rectangle(x1, y1, x2, y2, color);
            }
        }
    }

    fn draw_cage_glyphs(&mut self) {
        let scale = Scale::uniform(self.cell_width as f32 * 0.25);
        let v_metrics = self.font.v_metrics(scale);

        for cage in self.puzzle.cages() {
            let text = match cage.operator().symbol() {
                Some(symbol) => format!("{}{}", cage.target(), symbol),
                None => cage.target().to_string(),
            };

            let pos = cage.cells().min_by_key(|cell| cell.id()).unwrap().coord();

            let pad = self.cell_width / 16;
            let offset = point(
                ((pos.col() as u32 * self.cell_width) + self.border_width + pad) as f32,
                ((pos.row() as u32 * self.cell_width) + self.border_width + pad) as f32
                    + v_metrics.ascent,
            );

            for glyph in self.font.layout(&text, scale, offset).collect::<Vec<_>>() {
                self.overlay_glyph(&glyph);
            }
        }
    }

    fn highlight_cells(&mut self) {
        let cell_changes = match self.cell_changes {
            None => return,
            Some(cell_changes) => cell_changes,
        };
        for (&id, _) in cell_changes {
            let coord = self.puzzle.coord_at(id);
            self.draw_rectangle(
                coord.col() as u32 * self.cell_width + self.border_width,
                coord.row() as u32 * self.cell_width + self.border_width,
                (coord.col() + 1) as u32 * self.cell_width,
                (coord.row() + 1) as u32 * self.cell_width,
                COLOR_HIGHLIGHT_BG,
            );
        }
    }

    fn draw_markup(&mut self, cell_variables: &Square<CellVariable>) {
        for (id, cell) in cell_variables.iter().enumerate() {
            let cell_change = self
                .cell_changes
                .and_then(|cell_changes| cell_changes.get(id));
            let domain_removals: Option<(&ValueSet, AHashSet<i32>)> = match cell {
                CellVariable::Unsolved(domain) => {
                    let removals = match cell_change {
                        Some(CellChange::DomainRemovals(ref values)) => {
                            values.iter().copied().collect()
                        }
                        Some(CellChange::Solution(_)) => domain.iter().collect(),
                        _ => AHashSet::default(),
                    };
                    Some((domain, removals))
                }
                _ => None,
            };
            let solution = if let CellVariable::Solved(value) = *cell {
                Some(value)
            } else if let Some(&CellChange::Solution(value)) = cell_change {
                Some(value)
            } else {
                match domain_removals {
                    Some((ref domain, ref removals)) if domain.len() - removals.len() == 1 => {
                        // since there is one domain value left, show the solution
                        Some(domain.iter().find(|v| !removals.contains(v)).unwrap())
                    }
                    _ => None,
                }
            };
            let pos = cell_variables.coord_at(id);
            if let Some(value) = solution {
                self.draw_cell_solution(pos, value);
            }
            if let Some((domain, removals)) = domain_removals {
                self.draw_domain(pos, &domain, &removals);
            }
        }
    }

    fn draw_solution(&mut self, solution: &Square<Value>) {
        for (pos, value) in solution.iter_coord() {
            self.draw_cell_solution(pos, *value)
        }
    }

    fn draw_domain(&mut self, pos: Coord, domain: &ValueSet, removals: &AHashSet<i32>) {
        // the maximum number of characters that can fit on one line in a cell
        const MAX_LINE_LEN: u32 = 5;

        let scale = Scale::uniform(self.cell_width as f32 * 0.2);
        let v_metrics = self.font.v_metrics(scale);

        if domain.len() as u32 > MAX_LINE_LEN * 2 {
            return;
        }
        let mut char_x = 0;
        let mut char_y = 0;
        for n in domain {
            let s = n.to_string();
            let mut chars = s.chars();
            let c = chars.next().unwrap();
            if chars.next().is_some() {
                panic!("Expected a single char in {}", s)
            }
            let point = point(
                (pos.col() as u32 * self.cell_width + self.border_width + 1) as f32
                    + char_x as f32 * v_metrics.ascent,
                ((pos.row() as u32 + 1) * self.cell_width - 2) as f32
                    - char_y as f32 * v_metrics.ascent,
            );
            let glyph = self.font.glyph(c).scaled(scale).positioned(point);
            self.overlay_glyph(&glyph);
            if removals.contains(&n) {
                self.overlay_glyph_color(
                    &self.font.glyph('/').scaled(scale).positioned(point),
                    RED,
                );
            }
            char_x += 1;
            if char_x == MAX_LINE_LEN {
                char_x = 0;
                char_y += 1;
            }
        }
    }

    fn draw_cell_solution(&mut self, pos: Coord, value: i32) {
        let scale = Scale::uniform(self.cell_width as f32 * 0.8);
        let v_metrics = self.font.v_metrics(scale);

        let s = value.to_string();
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() {
            panic!("{} has too many characters", value)
        }
        let glyph = self.font.glyph(c).scaled(scale);
        let h_metrics = glyph.h_metrics();
        let x = (pos.col() as u32 * self.cell_width + self.border_width) as f32
            + ((self.cell_width - self.border_width) as f32 - h_metrics.advance_width) / 2.0;
        let y = ((pos.row() as u32 + 1) * self.cell_width) as f32
            - ((self.cell_width - self.border_width) as f32 - v_metrics.ascent) / 2.0;
        let point = point(x, y);
        let glyph = glyph.positioned(point);
        self.overlay_glyph(&glyph);
    }

    fn draw_rectangle(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb) {
        for x in x1..x2 {
            for y in y1..y2 {
                self.buffer.put_pixel(x, y, color);
            }
        }
    }

    fn overlay_glyph(&mut self, glyph: &PositionedGlyph<'_>) {
        let bb = glyph.pixel_bounding_box().unwrap();
        glyph.draw(|x, y, v| {
            if v == 0.0 {
                return;
            };
            self.buffer
                .get_pixel_mut(bb.min.x as u32 + x, bb.min.y as u32 + y)
                .apply(|c| (f32::from(c) * (1.0 - v)) as u8);
        });
    }

    fn overlay_glyph_color(&mut self, glyph: &PositionedGlyph<'_>, color: Rgb) {
        let bb = glyph.pixel_bounding_box().unwrap();
        glyph.draw(|x, y, v| {
            if v == 0.0 {
                return;
            };
            let pixel = self
                .buffer
                .get_pixel_mut(bb.min.x as u32 + x, bb.min.y as u32 + y);
            pixel.apply2(&color, |a, b| min(a, b) + (max(a, b) - min(a, b)) / 2);
        });
    }
}
