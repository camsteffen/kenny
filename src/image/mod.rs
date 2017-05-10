extern crate png;
extern crate image;

pub use self::image::RgbImage;
pub use self::image::Rgb;

pub const BLACK: Rgb<u8> = Rgb { data: [0; 3] };
pub const WHITE: Rgb<u8> = Rgb { data: [255; 3] };

pub const COLOR_CELL_BORDER:  Rgb<u8> = Rgb { data: [205; 3] };
pub const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
pub const COLOR_BG: Rgb<u8> = WHITE;

use cell_domain::CellDomain;
use puzzle::{Puzzle, Cage};
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::point;
use solver::Solver;
use square::Coord;
use std::io;
use variable::Variable;

pub trait AsImage {
    fn as_image(&self) -> RgbImage;
}

pub struct PuzzleImageInfo<'a> {
    pub cell_width: u32,
    pub border_width: u32,
    pub image_width: u32,
    pub font: Font<'a>,
}

impl<'a> PuzzleImageInfo<'a> {
    pub fn from_puzzle_size_default(puzzle_size: usize) -> PuzzleImageInfo<'a> {
        let cell_width = 60 as u32;
        let border_width = cell_width / 25;
        let image_width = cell_width * puzzle_size as u32 + border_width;

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
}

pub fn draw_rectangle(image: &mut RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb<u8>) {
    for x in x1..x2 {
        for y in y1..y2 {
            image.put_pixel(x, y, color);
        }
    }
}

pub fn draw_grid(
    image: &mut RgbImage,
    info: &PuzzleImageInfo,
    puzzle: &Puzzle)
{
    let image_width = info.cell_width * puzzle.size as u32 + info.border_width;
    let cells_width = info.cell_width * puzzle.size as u32;

    // draw outer border
    draw_rectangle(image, 0, 0, cells_width, info.border_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, info.border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, 0, info.border_width, info.border_width, image_width, COLOR_CAGE_BORDER);

    let cage_map = puzzle.cage_map();

    // draw horizontal line segments
    for i in 1..puzzle.size { // row
        for j in 0..puzzle.size { // col
            let pos1 = Coord([i - 1, j]);
            let pos2 = Coord([i, j]);
            let color = if cage_map[pos1] == cage_map[pos2] {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width + info.border_width;
            let y1 = i as u32 * info.cell_width;
            let x2 = x1 + info.cell_width - info.border_width;
            let y2 = y1 + info.border_width;
            draw_rectangle(image, x1, y1, x2, y2, color);
        }
    }
    // draw vertical line segments
    for i in 0..puzzle.size { // row
        for j in 1..puzzle.size { // col
            let pos1 = Coord([i, j - 1]);
            let pos2 = Coord([i, j]);
            let color = if cage_map[pos1] == cage_map[pos2] {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width;
            let y1 = i as u32 * info.cell_width + info.border_width;
            let x2 = x1 + info.border_width;
            let y2 = y1 + info.cell_width - info.border_width;
            draw_rectangle(image, x1, y1, x2, y2, color);
        }
    }

    // draw intersections
    for i in 1..puzzle.size {
        for j in 1..puzzle.size {
            let first = cage_map[Coord([i - 1, j - 1])];
            let pos = [
                Coord([i - 1, j]),
                Coord([i, j - 1]),
                Coord([i, j]),
            ];
            let color = if pos.iter().all(|pos| cage_map[*pos] == first) {
                COLOR_CELL_BORDER
            } else {
                COLOR_CAGE_BORDER
            };
            let x1 = j as u32 * info.cell_width;
            let y1 = i as u32 * info.cell_width;
            let x2 = x1 + info.border_width;
            let y2 = y1 + info.border_width;
            draw_rectangle(image, x1, y1, x2, y2, color);
        }
    }
}

pub fn draw_cage_glyphs(
    image: &mut RgbImage,
    info: &PuzzleImageInfo,
    puzzle: &Puzzle)
{
    let scale = Scale::uniform(info.cell_width as f32 * 0.25);
    let v_metrics = info.font.v_metrics(scale);

    for cage in puzzle.cages {
        let text = &format!("{}{}", cage.operator.symbol(), cage.target);

        let index = *cage.cells.iter().min().unwrap();
        let pos = Coord::from_index(index, puzzle.size);

        let offset = point(
            ((pos[1] as u32 * info.cell_width) + info.border_width) as f32,
            ((pos[0] as u32 * info.cell_width) + info.border_width) as f32 + v_metrics.ascent);

        for glyph in info.font.layout(text, scale, offset) {
            let bb = glyph.pixel_bounding_box().expect("glyph bounding box");
            glyph.draw(|x, y, v| {
                if v == 0.0 { return };
                let v = ((1.0 - v) * 255.0) as u8;
                image.put_pixel(
                    bb.min.x as u32 + x,
                    bb.min.y as u32 + y,
                    Rgb { data: [v; 3] });
            });
        }
    }
}
