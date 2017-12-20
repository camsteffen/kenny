//! Generate images for unsolved or solved puzzles

extern crate png;
extern crate image;

use self::image::{Rgb, RgbImage};

const BLACK: Rgb<u8> = Rgb { data: [0; 3] };
const WHITE: Rgb<u8> = Rgb { data: [255; 3] };

const COLOR_CELL_BORDER:  Rgb<u8> = Rgb { data: [205; 3] };
const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
const COLOR_BG: Rgb<u8> = WHITE;

use solve::CellDomain;
use puzzle::Puzzle;
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::point;
use solve::Solver;
use solve::CellVariable;
use collections::square::Coord;

/// The `AsImage` trait is uesd to generate an in-memory image
pub trait AsImage {

    /// Generate an image representation
    fn as_image(&self) -> RgbImage;
}

struct PuzzleImageInfo<'a> {
    pub cell_width: u32,
    pub border_width: u32,
    pub image_width: u32,
    pub font: Font<'a>,
}

impl<'a> PuzzleImageInfo<'a> {
    pub fn from_puzzle_size_default(puzzle_size: u32) -> PuzzleImageInfo<'a> {
        let cell_width = 60;
        let border_width = cell_width / 25;
        let image_width = cell_width * puzzle_size + border_width;

        let font_data = include_bytes!("/Library/Fonts/Verdana.ttf");
        let font_collection = FontCollection::from_bytes(font_data as &[u8]);
        let font = font_collection.font_at(0).expect("load font");

        PuzzleImageInfo {
            cell_width: cell_width,
            border_width: border_width,
            image_width: image_width,
            font: font,
        }
    }

    pub fn buffer(&self) -> RgbImage {
        RgbImage::from_pixel(self.image_width, self.image_width, COLOR_BG)
    }
}

impl AsImage for Puzzle {
    fn as_image(&self) -> RgbImage {
        let info = PuzzleImageInfo::from_puzzle_size_default(self.size as u32);
        let mut buffer = info.buffer();
        render_puzzle(&mut buffer, &info, self);
        buffer
    }
}

impl<'a> AsImage for Solver<'a> {
    fn as_image(&self) -> RgbImage {
        let info = PuzzleImageInfo::from_puzzle_size_default(self.puzzle.size as u32);
        let mut buffer = info.buffer();
        render_puzzle(&mut buffer, &info, self.puzzle);
        render_solver(&mut buffer, &info, self);
        buffer
    }
}

fn render_puzzle(buffer: &mut RgbImage, info: &PuzzleImageInfo, puzzle: &Puzzle) {
    draw_grid(buffer, info, puzzle);
    draw_cage_glyphs(buffer, info, puzzle);
}

fn render_solver(buffer: &mut RgbImage, info: &PuzzleImageInfo, solver: &Solver) {
    draw_markup(buffer, info, solver);
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
    let image_width = info.cell_width * puzzle.size as u32 + info.border_width;
    let cells_width = info.cell_width * puzzle.size as u32;

    // draw outer border
    draw_rectangle(buffer, 0, 0, cells_width, info.border_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, info.border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
    draw_rectangle(buffer, 0, info.border_width, info.border_width, image_width, COLOR_CAGE_BORDER);

    // draw horizontal line segments
    for i in 1..puzzle.size as usize { // row
        for j in 0..puzzle.size as usize { // col
            let pos1 = Coord([i - 1, j]);
            let pos2 = Coord([i, j]);
            let color = if puzzle.cage_at(pos1) == puzzle.cage_at(pos2) {
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
    for i in 0..puzzle.size as usize { // row
        for j in 1..puzzle.size as usize { // col
            let pos1 = Coord([i, j - 1]);
            let pos2 = Coord([i, j]);
            let color = if puzzle.cage_at(pos1) == puzzle.cage_at(pos2) {
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
    for i in 1..puzzle.size as usize {
        for j in 1..puzzle.size as usize {
            let first = puzzle.cage_at(Coord([i - 1, j - 1]));
            let pos = [
                Coord([i - 1, j]),
                Coord([i, j - 1]),
                Coord([i, j]),
            ];
            let color = if pos.iter().all(|pos| puzzle.cage_at(*pos) == first) {
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
        let pos = index.as_coord(puzzle.size as usize);

        let pad = info.cell_width / 16;
        let offset = point(
            ((pos[1] as u32 * info.cell_width) + info.border_width + pad) as f32,
            ((pos[0] as u32 * info.cell_width) + info.border_width + pad) as f32 + v_metrics.ascent);

        for glyph in info.font.layout(&text, scale, offset) {
            let bb = glyph.pixel_bounding_box().expect("glyph bounding box");
            glyph.draw(|x, y, v| {
                if v == 0.0 { return };
                let v = ((1.0 - v) * 255.0) as u8;
                buffer.put_pixel(
                    bb.min.x as u32 + x,
                    bb.min.y as u32 + y,
                    Rgb { data: [v; 3] });
            });
        }
    }
}

fn draw_markup(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    solver: &Solver)
{
    for (pos, cell) in solver.cells.iter_coord() {
        match *cell {
            CellVariable::Unsolved(ref domain) => {
                draw_range_domain(buffer, info, pos, domain)
            },
            CellVariable::Solved(value) => {
                draw_cell_solution(buffer, info, pos, value)
            },
        };
    }
}

fn draw_range_domain(
    buffer: &mut RgbImage,
    info: &PuzzleImageInfo,
    pos: Coord,
    domain: &CellDomain)
{
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
        let bb = glyph.pixel_bounding_box().unwrap();
        glyph.draw(|x, y, v| {
            if v == 0.0 { return };
            let v = ((1.0 - v) * 255.0) as u8;
            buffer.put_pixel(
                bb.min.x as u32 + x,
                bb.min.y as u32 + y,
                Rgb { data: [v; 3] });
        });
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
    let bb = glyph.pixel_bounding_box().unwrap();
    glyph.draw(|x, y, val| {
        if val == 0.0 { return };
        let val = ((1.0 - val) * 255.0) as u8;
        buffer.put_pixel(bb.min.x as u32 + x,
                        bb.min.y as u32 + y,
                        Rgb { data: [val; 3] });
    });
}

