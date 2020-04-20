//! Generate images for unsolved or solved puzzles

extern crate rusttype;

const BLACK: Rgb<u8> = Rgb([0; 3]);
const WHITE: Rgb<u8> = Rgb([255; 3]);

const COLOR_CELL_BORDER:  Rgb<u8> = Rgb([205; 3]);
const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
const COLOR_BG: Rgb<u8> = WHITE;
const COLOR_HIGHLIGHT_BG: Rgb<u8> = Rgb([255, 255, 150]);

const BORDER_CELL_RATIO: u32 = 25;
const DEFAULT_CELL_WIDTH: u32 = 60;

use crate::collections::Square;
use crate::collections::square::{Coord, AsSquareIndex};
use crate::collections::square::SquareIndex;
use fnv::FnvHashSet;
use image::Pixel;
use image::{Rgb, RgbImage};
use crate::puzzle::Puzzle;
use crate::puzzle::solve::CellDomain;
use crate::puzzle::solve::CellVariable;
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::point;
use rusttype::PositionedGlyph;

pub struct PuzzleImageBuilder<'a> {
    puzzle: &'a Puzzle,

    cell_variables: Option<&'a Square<CellVariable>>,
    highlighted_cells: Option<&'a [SquareIndex]>,

    image_width: u32,
    cell_width: u32,
    border_width: u32,

    font: Font<'a>,
}

impl<'a> PuzzleImageBuilder<'a> {

    pub fn new(puzzle: &'a Puzzle) -> Self {
        let font_data = include_bytes!("../../res/Roboto-Regular.ttf");
        let font_collection = FontCollection::from_bytes(font_data as &[u8]).expect("Error loading font");
        let font = font_collection.font_at(0).expect("load font");

        let cell_width = DEFAULT_CELL_WIDTH;
        let border_width = Self::calc_border_width(cell_width);
        let image_width = Self::calc_image_width(cell_width, puzzle.width(), border_width);

        Self {
            puzzle,
            image_width,
            border_width,
            cell_width,
            font,
            cell_variables: None,
            highlighted_cells: None,
        }
    }

    pub fn cell_width(&mut self, cell_width: u32) -> &mut Self {
        self.cell_width = cell_width;
        self.border_width = Self::calc_border_width(cell_width);
        self.image_width = Self::calc_image_width(cell_width, self.puzzle.width(), self.border_width);
        self
    }

    pub fn image_width(&mut self, image_width: u32) -> &mut Self {
        let a = BORDER_CELL_RATIO.checked_mul(image_width).expect("image width is too big");
        let b = BORDER_CELL_RATIO * self.puzzle.width() as u32 + 1;
        self.cell_width(a / b)
    }

    pub fn highlighted_cells(&mut self, highlighted_cells: Option<&'a [SquareIndex]>) -> &mut Self {
        self.highlighted_cells = highlighted_cells;
        self
    }

    pub fn cell_variables(&mut self, cell_variables: Option<&'a Square<CellVariable>>) -> &mut Self {
        self.cell_variables = cell_variables;
        self
    }

    pub fn build(&self) -> RgbImage {
        let mut buffer = match self.highlighted_cells {
            Some(highlighted_cells) =>
                Self::buffer_with_highlighted_cells(highlighted_cells, self.image_width, self.cell_width, self.puzzle.width()),
            None => RgbImage::from_pixel(self.image_width, self.image_width, COLOR_BG),
        };
        self.draw_grid(&mut buffer);
        self.draw_cage_glyphs(&mut buffer);
        if let Some(cell_variables) = self.cell_variables {
            self.draw_markup(&mut buffer, cell_variables);
        }
        buffer
    }

    fn calc_border_width(cell_width: u32) -> u32 {
        cell_width / BORDER_CELL_RATIO
    }

    fn calc_image_width(cell_width: u32, puzzle_width: usize, border_width: u32) -> u32 {
        cell_width * puzzle_width as u32 + border_width
    }

    fn buffer_with_highlighted_cells(highlighted_cells: &[SquareIndex], image_width: u32, cell_width: u32,
                                     puzzle_width: usize) -> RgbImage {
        let highlighted_cells = highlighted_cells.iter().cloned().collect::<FnvHashSet<_>>();
        RgbImage::from_fn(image_width, image_width, |i, j| {
            let index = Coord([(j / cell_width) as usize, (i / cell_width) as usize])
                .as_index(puzzle_width);
            if highlighted_cells.contains(&index) {
                COLOR_HIGHLIGHT_BG
            } else {
                COLOR_BG
            }
        })
    }

    fn draw_grid(&self, buffer: &mut RgbImage) {
        let image_width = self.cell_width * self.puzzle.width() as u32 + self.border_width;
        let cells_width = self.cell_width * self.puzzle.width() as u32;

        // draw outer border
        draw_rectangle(buffer, 0, 0, cells_width, self.border_width, COLOR_CAGE_BORDER);
        draw_rectangle(buffer, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
        draw_rectangle(buffer, self.border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
        draw_rectangle(buffer, 0, self.border_width, self.border_width, image_width, COLOR_CAGE_BORDER);

        // draw horizontal line segments
        for i in 1..self.puzzle.width() { // row
            for j in 0..self.puzzle.width() { // col
                let pos1 = Coord([i - 1, j]);
                let pos2 = Coord([i, j]);
                let color = if self.puzzle.cell(pos1).cage().index() == self.puzzle.cell(pos2).cage().index() {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width + self.border_width;
                let y1 = i as u32 * self.cell_width;
                let x2 = x1 + self.cell_width - self.border_width;
                let y2 = y1 + self.border_width;
                draw_rectangle(buffer, x1, y1, x2, y2, color);
            }
        }

        // draw vertical line segments
        for i in 0..self.puzzle.width() { // row
            for j in 1..self.puzzle.width() { // col
                let pos1 = Coord([i, j - 1]);
                let pos2 = Coord([i, j]);
                let color = if self.puzzle.cell(pos1).cage().index() == self.puzzle.cell(pos2).cage().index() {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width;
                let y1 = i as u32 * self.cell_width + self.border_width;
                let x2 = x1 + self.border_width;
                let y2 = y1 + self.cell_width - self.border_width;
                draw_rectangle(buffer, x1, y1, x2, y2, color);
            }
        }

        // draw intersections
        for i in 1..self.puzzle.width() {
            for j in 1..self.puzzle.width() {
                let first = self.puzzle.cell(Coord([i - 1, j - 1])).cage().index();
                let pos = [
                    Coord([i - 1, j]),
                    Coord([i, j - 1]),
                    Coord([i, j]),
                ];
                let color = if pos.iter().all(|pos| self.puzzle.cell(*pos).cage().index() == first) {
                    COLOR_CELL_BORDER
                } else {
                    COLOR_CAGE_BORDER
                };
                let x1 = j as u32 * self.cell_width;
                let y1 = i as u32 * self.cell_width;
                let x2 = x1 + self.border_width;
                let y2 = y1 + self.border_width;
                draw_rectangle(buffer, x1, y1, x2, y2, color);
            }
        }
    }

    fn draw_cage_glyphs(&self, buffer: &mut RgbImage) {
        let scale = Scale::uniform(self.cell_width as f32 * 0.25);
        let v_metrics = self.font.v_metrics(scale);

        for cage in self.puzzle.cages().iter() {
            let text = match cage.operator().symbol() {
                Some(symbol) => format!("{}{}", cage.target(), symbol),
                None => cage.target().to_string(),
            };

            let pos = *cage.cells().min_by_key(|cell| cell.index()).unwrap().coord();

            let pad = self.cell_width / 16;
            let offset = point(
                ((pos[1] as u32 * self.cell_width) + self.border_width + pad) as f32,
                ((pos[0] as u32 * self.cell_width) + self.border_width + pad) as f32 + v_metrics.ascent);

            for glyph in self.font.layout(&text, scale, offset) {
                overlay_glyph(buffer, &glyph);
            }
        }
    }

    fn draw_markup(&self, buffer: &mut RgbImage, cell_variables: &Square<CellVariable>) {
        for (pos, cell) in cell_variables.iter_coord() {
            match cell {
                CellVariable::Unsolved(domain) => {
                    self.draw_domain(buffer, pos, domain)
                },
                CellVariable::Solved(value) => {
                    self.draw_cell_solution(buffer, pos, *value)
                },
            };
        }
    }

    fn draw_domain(&self, buffer: &mut RgbImage, pos: Coord, domain: &CellDomain) {
        // the maximum number of characters that can fit on one line in a cell
        const MAX_LINE_LEN: u32 = 5;

        let scale = Scale::uniform(self.cell_width as f32 * 0.2);
        let v_metrics = self.font.v_metrics(scale);

        if domain.len() as u32 > MAX_LINE_LEN * 2 { return }
        let mut char_x = 0;
        let mut char_y = 0;
        for n in domain {
            let s = n.to_string();
            let mut chars = s.chars();
            let c = chars.next().unwrap();
            if chars.next().is_some() { panic!("Expected a single char in {}", s) }
            let point = point(
                (pos[1] as u32 * self.cell_width + self.border_width + 1) as f32 + char_x as f32 * v_metrics.ascent,
                ((pos[0] as u32 + 1) * self.cell_width - 2) as f32 - char_y as f32 * v_metrics.ascent);
            let glyph = self.font.glyph(c)
                .scaled(scale)
                .positioned(point);
            overlay_glyph(buffer, &glyph);
            char_x += 1;
            if char_x == MAX_LINE_LEN {
                char_x = 0;
                char_y += 1;
            }
        }
    }

    fn draw_cell_solution(&self, buffer: &mut RgbImage, pos: Coord, value: i32) {
        let scale = Scale::uniform(self.cell_width as f32 * 0.8);
        let v_metrics = self.font.v_metrics(scale);

        let s = value.to_string();
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() { panic!("{} has too many characters", value) }
        let glyph = self.font.glyph(c)
            .scaled(scale);
        let h_metrics = glyph.h_metrics();
        let x = (pos[1] as u32 * self.cell_width + self.border_width) as f32 + ((self.cell_width - self.border_width) as f32 - h_metrics.advance_width) / 2.0;
        let y = ((pos[0] as u32 + 1) * self.cell_width) as f32 - ((self.cell_width - self.border_width) as f32 - v_metrics.ascent) / 2.0;
        let point = point(x, y);
        let glyph = glyph.positioned(point);
        overlay_glyph(buffer, &glyph);
    }

}

fn draw_rectangle(buffer: &mut RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb<u8>) {
    for x in x1..x2 {
        for y in y1..y2 {
            buffer.put_pixel(x, y, color);
        }
    }
}

fn overlay_glyph(buffer: &mut RgbImage, glyph: &PositionedGlyph) {
    let bb = glyph.pixel_bounding_box().unwrap();
    glyph.draw(|x, y, v| {
        if v == 0.0 { return };
        buffer.get_pixel_mut(bb.min.x as u32 + x, bb.min.y as u32 + y)
            .apply(|c| (f32::from(c) * (1.0 - v)) as u8);
    });
}
