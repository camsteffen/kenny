//! Generate images for unsolved or solved puzzles

extern crate rusttype;

const BLACK: Rgb<u8> = Rgb { data: [0; 3] };
const WHITE: Rgb<u8> = Rgb { data: [255; 3] };

const COLOR_CELL_BORDER:  Rgb<u8> = Rgb { data: [205; 3] };
const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
const COLOR_BG: Rgb<u8> = WHITE;
const COLOR_HIGHLIGHT_BG: Rgb<u8> = Rgb { data: [255, 255, 150] };

const BORDER_CELL_RATIO: u32 = 25;
const DEFAULT_CELL_WIDTH: u32 = 60;

use collections::Square;
use collections::square::Coord;
use collections::square::SquareIndex;
use fnv::FnvHashSet;
use image::Pixel;
use image::{Rgb, RgbImage};
use puzzle::Puzzle;
use puzzle::solve::CellDomain;
use puzzle::solve::CellVariable;
use puzzle::solve::markup::PuzzleMarkup;
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::point;
use rusttype::PositionedGlyph;

pub fn puzzle_image(puzzle: &Puzzle) -> RgbImage {
    let info = PuzzleImageInfo::from_puzzle(puzzle);
    let mut buffer = info.create_buffer();
    render_puzzle(&mut buffer, &info, puzzle);
    buffer
}

pub fn puzzle_image_with_markup(puzzle: &Puzzle, puzzle_markup: &PuzzleMarkup) -> RgbImage {
    let info = PuzzleImageInfo::from_puzzle(puzzle);
    let mut buffer = info.create_buffer();
    render_puzzle(&mut buffer, &info, puzzle);
    render_markup(&mut buffer, &info, &puzzle_markup.cell_variables);
    buffer
}

pub fn puzzle_image_with_markup_and_highlighted_cells(puzzle: &Puzzle, puzzle_markup: &PuzzleMarkup,
                                                      highlighted_cells: &[SquareIndex]) -> RgbImage {
    let info = PuzzleImageInfo::from_puzzle(puzzle);
    let mut buffer = info.create_buffer_with_highlighted_cells(puzzle.width, highlighted_cells);
    render_puzzle(&mut buffer, &info, puzzle);
    render_markup(&mut buffer, &info, &puzzle_markup.cell_variables);
    buffer
}

struct PuzzleImageInfo<'a> {
    pub cell_width: u32,
    pub border_width: u32,
    pub image_width: u32,
    pub font: Font<'a>,
}

impl<'a> PuzzleImageInfo<'a> {

    pub fn from_puzzle(puzzle: &Puzzle) -> PuzzleImageInfo<'a> {
        Self::from_puzzle_and_cell_width(puzzle, DEFAULT_CELL_WIDTH)
    }

    pub fn from_puzzle_and_cell_width(puzzle: &Puzzle, cell_width: u32) -> PuzzleImageInfo<'a> {
        let border_width = cell_width / BORDER_CELL_RATIO;
        let image_width = cell_width * puzzle.width as u32 + border_width;

        let font_data = include_bytes!("/Library/Fonts/Verdana.ttf");
        let font_collection = FontCollection::from_bytes(font_data as &[u8]);
        let font = font_collection.font_at(0).expect("load font");

        PuzzleImageInfo {
            cell_width,
            border_width,
            image_width,
            font,
        }
    }

    pub fn from_puzzle_and_image_width(puzzle: &Puzzle, image_width: u32) -> PuzzleImageInfo<'a> {
        let a = BORDER_CELL_RATIO.checked_mul(image_width).expect("image width is too big");
        let b = BORDER_CELL_RATIO * puzzle.width as u32 + 1;
        let cell_width = a / b;
        Self::from_puzzle_and_cell_width(puzzle, cell_width)
    }

    pub fn create_buffer(&self) -> RgbImage {
        RgbImage::from_pixel(self.image_width, self.image_width, COLOR_BG)
    }

    pub fn create_buffer_with_highlighted_cells(&self, puzzle_width: usize, highlighted_cells: &[SquareIndex]) -> RgbImage {
        let highlighted_cells = highlighted_cells.iter().cloned().collect::<FnvHashSet<_>>();
        RgbImage::from_fn(self.image_width, self.image_width, |i, j| {
            let index = Coord([(j / self.cell_width) as usize, (i / self.cell_width) as usize])
                .as_index(puzzle_width);
            if highlighted_cells.contains(&index) {
                COLOR_HIGHLIGHT_BG
            } else {
                COLOR_BG
            }
        })
    }
}

fn render_puzzle(buffer: &mut RgbImage, info: &PuzzleImageInfo, puzzle: &Puzzle) {
    draw_grid(buffer, info, puzzle);
    draw_cage_glyphs(buffer, info, puzzle);
}

fn render_markup(buffer: &mut RgbImage, info: &PuzzleImageInfo, cell_variables: &Square<CellVariable>) {
    draw_markup(buffer, info, cell_variables);
}

fn draw_rectangle(buffer: &mut RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb<u8>) {
    for x in x1..x2 {
        for y in y1..y2 {
            buffer.put_pixel(x, y, color);
        }
    }
}

fn draw_grid(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    puzzle: &Puzzle)
{
    let image_width = info.cell_width * puzzle.width as u32 + info.border_width;
    let cells_width = info.cell_width * puzzle.width as u32;

    // draw outer border
    draw_rectangle(buffer, 0, 0, cells_width, info.border_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, info.border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, 0, info.border_width, info.border_width, image_width, COLOR_CAGE_BORDER);

    // draw horizontal line segments
    for i in 1..puzzle.width as usize { // row
        for j in 0..puzzle.width as usize { // col
            let pos1 = Coord([i - 1, j]);
            let pos2 = Coord([i, j]);
            let color = if puzzle.cage_index_at(pos1) == puzzle.cage_index_at(pos2) {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width + info.border_width;
            let y1 = i as u32 * info.cell_width;
            let x2 = x1 + info.cell_width - info.border_width;
            let y2 = y1 + info.border_width;
            draw_rectangle(buffer, x1, y1, x2, y2, color);
        }
    }
    // draw vertical line segments
    for i in 0..puzzle.width as usize { // row
        for j in 1..puzzle.width as usize { // col
            let pos1 = Coord([i, j - 1]);
            let pos2 = Coord([i, j]);
            let color = if puzzle.cage_index_at(pos1) == puzzle.cage_index_at(pos2) {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width;
            let y1 = i as u32 * info.cell_width + info.border_width;
            let x2 = x1 + info.border_width;
            let y2 = y1 + info.cell_width - info.border_width;
            draw_rectangle(buffer, x1, y1, x2, y2, color);
        }
    }

    // draw intersections
    for i in 1..puzzle.width as usize {
        for j in 1..puzzle.width as usize {
            let first = puzzle.cage_index_at(Coord([i - 1, j - 1]));
            let pos = [
                Coord([i - 1, j]),
                Coord([i, j - 1]),
                Coord([i, j]),
            ];
            let color = if pos.iter().all(|pos| puzzle.cage_index_at(*pos) == first) {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width;
            let y1 = i as u32 * info.cell_width;
            let x2 = x1 + info.border_width;
            let y2 = y1 + info.border_width;
            draw_rectangle(buffer, x1, y1, x2, y2, color);
        }
    }
}

fn draw_cage_glyphs(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    puzzle: &Puzzle)
{
    let scale = Scale::uniform(info.cell_width as f32 * 0.25);
    let v_metrics = info.font.v_metrics(scale);

    for cage in &puzzle.cages {
        let text = match cage.operator.symbol() {
            Some(symbol) => format!("{}{}", cage.target, symbol),
            None => cage.target.to_string(),
        };

        let index = *cage.cells.iter().min().unwrap();
        let pos = index.as_coord(puzzle.width as usize);

        let pad = info.cell_width / 16;
        let offset = point(
            ((pos[1] as u32 * info.cell_width) + info.border_width + pad) as f32,
            ((pos[0] as u32 * info.cell_width) + info.border_width + pad) as f32 + v_metrics.ascent);

        for glyph in info.font.layout(&text, scale, offset) {
            overlay_glyph(buffer, glyph);
        }
    }
}

fn draw_markup(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    cell_variables: &Square<CellVariable>)
{
    for (pos, cell) in cell_variables.iter_coord() {
        match cell {
            CellVariable::Unsolved(domain) => {
                draw_domain(buffer, info, pos, domain)
            },
            CellVariable::Solved(value) => {
                draw_cell_solution(buffer, info, pos, *value)
            },
        };
    }
}

fn draw_domain(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    pos: Coord,
    domain: &CellDomain)
{
    // the maximum number of characters that can fit on one line in a cell
    const MAX_LINE_LEN: u32 = 5;

    let scale = Scale::uniform(info.cell_width as f32 * 0.2);
    let v_metrics = info.font.v_metrics(scale);

    if domain.len() as u32 > MAX_LINE_LEN * 2 { return }
    let mut char_x = 0;
    let mut char_y = 0;
    for n in domain {
        let s = n.to_string();
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() { panic!("Expected a single char in {}", s) }
        let point = point(
            ((pos[1] as u32 * info.cell_width + info.border_width + 1) as f32 + char_x as f32 * v_metrics.ascent),
            ((pos[0] as u32 + 1) * info.cell_width - 2) as f32 - char_y as f32 * v_metrics.ascent);
        let glyph = info.font.glyph(c).expect(&format!("No glyph for {}", c))
            .scaled(scale)
            .positioned(point);
        overlay_glyph(buffer, glyph);
        char_x += 1;
        if char_x == MAX_LINE_LEN {
            char_x = 0;
            char_y += 1;
        }
    }
}

fn draw_cell_solution(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    pos: Coord,
    value: i32)
{
    let scale = Scale::uniform(info.cell_width as f32 * 0.8);
    let v_metrics = info.font.v_metrics(scale);

    let s = value.to_string();
    let mut chars = s.chars();
    let c = chars.next().unwrap();
    if let Some(c2) = chars.next() { panic!("Unexpected char: {}", c2) }
    let glyph = info.font.glyph(c).unwrap_or_else(|| panic!("No glyph for {}", c))
        .scaled(scale);
    let h_metrics = glyph.h_metrics();
    let x = (pos[1] as u32 * info.cell_width + info.border_width) as f32 + ((info.cell_width - info.border_width) as f32 - h_metrics.advance_width) / 2.0;
    let y = ((pos[0] as u32 + 1) * info.cell_width) as f32 - ((info.cell_width - info.border_width) as f32 - v_metrics.ascent) / 2.0;
    let point = point(x, y);
    let glyph = glyph.positioned(point);
    overlay_glyph(buffer, glyph);
}

fn overlay_glyph(buffer: &mut RgbImage, glyph: PositionedGlyph) {
    let bb = glyph.pixel_bounding_box().unwrap();
    glyph.draw(|x, y, v| {
        if v == 0.0 { return };
        buffer.get_pixel_mut(bb.min.x as u32 + x, bb.min.y as u32 + y)
            .apply(|c| (c as f32 * (1.0 - v)) as u8);
    });
}
