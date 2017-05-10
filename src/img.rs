extern crate png;
extern crate image;

use cell_domain::CellDomain;
use image::Rgb;
use image::RgbImage;
use puzzle::{Puzzle, Cage};
use rusttype::Font;
use rusttype::FontCollection;
use rusttype::Scale;
use rusttype::point;
use solver::Solver;
use square::Coord;
use std::io;
use variable::Variable;

const BLACK: Rgb<u8> = Rgb { data: [0; 3] };
const WHITE: Rgb<u8> = Rgb { data: [255; 3] };

const COLOR_CELL_BORDER:  Rgb<u8> = Rgb { data: [205; 3] };
const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
const COLOR_BG: Rgb<u8> = WHITE;

pub fn image(puzzle: &Puzzle, solver: Option<&Solver>, path: &str) -> Result<(), io::Error> {
    let cell_width = 60 as usize;
    let border_width = cell_width / 25;

    let image_width = (cell_width * puzzle.size + border_width) as u32;
    let mut image = RgbImage::from_pixel(image_width, image_width, COLOR_BG);

    draw_grid(&mut image, puzzle, cell_width as u32, border_width as u32);
    draw_cage_glyphs(&mut image, &puzzle.cages, solver, puzzle.size, cell_width, border_width);

    image.save(path)?;

    Ok(())
}

fn draw_rectangle(image: &mut RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: Rgb<u8>) {
    for x in x1..x2 {
        for y in y1..y2 {
            image.put_pixel(x, y, color);
        }
    }
}

fn draw_grid(
    image: &mut RgbImage,
    puzzle: &Puzzle,
    cell_width: u32,
    border_width: u32)
{
    let image_width = cell_width * puzzle.size as u32 + border_width;
    let cells_width = cell_width * puzzle.size as u32;

    // draw outer border
    draw_rectangle(image, 0, 0, cells_width, border_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, 0, border_width, border_width, image_width, COLOR_CAGE_BORDER);

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
            let x1 = j as u32 * cell_width + border_width;
            let y1 = i as u32 * cell_width;
            let x2 = x1 + cell_width - border_width;
            let y2 = y1 + border_width;
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
            let x1 = j as u32 * cell_width;
            let y1 = i as u32 * cell_width + border_width;
            let x2 = x1 + border_width;
            let y2 = y1 + cell_width - border_width;
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
            let x1 = j as u32 * cell_width;
            let y1 = i as u32 * cell_width;
            let x2 = x1 + border_width;
            let y2 = y1 + border_width;
            draw_rectangle(image, x1, y1, x2, y2, color);
        }
    }
}

fn draw_cage_glyphs(
    image: &mut RgbImage,
    cages: &[Cage],
    solver: Option<&Solver>,
    size: usize,
    cell_width: usize,
    border_width: usize)
{
    let font_data = include_bytes!("/Library/Fonts/Verdana.ttf");
    let collection = FontCollection::from_bytes(font_data as &[u8]);
    let font = collection.font_at(0).expect("load font");

    let scale = Scale::uniform(cell_width as f32 * 0.25);
    let v_metrics = font.v_metrics(scale);

    for cage in cages.iter() {
        let text = &format!("{}{}", cage.operator.symbol(), cage.target);

        let index = *cage.cells.iter().min().unwrap();
        let pos = Coord::from_index(index, size);

        let offset = point(
            ((pos[1] * cell_width) + border_width) as f32,
            ((pos[0] * cell_width) + border_width) as f32 + v_metrics.ascent);

        for glyph in font.layout(text, scale, offset) {
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

    // markup domain
    if let Some(solver) = solver {
        for (pos, cell) in solver.cells.iter_coord() {
            match *cell {
                Variable::Unsolved(ref domain) => {
                    draw_cell_domain(image, pos, domain, &font, cell_width, border_width)
                },
                Variable::Solved(value) => {
                    draw_cell_solution(image, pos, value, &font, cell_width, border_width)
                },
            };
        }
    }
}

fn draw_cell_domain(
    image: &mut RgbImage,
    pos: Coord,
    domain: &CellDomain,
    font: &Font,
    cell_width: usize,
    border_width: usize)
{
    const MAX_LINE_LEN: usize = 5;

    let scale = Scale::uniform(cell_width as f32 * 0.2);
    let v_metrics = font.v_metrics(scale);

    if domain.len() > MAX_LINE_LEN * 2 { return }
    let mut char_x = 0;
    let mut char_y = 0;
    for n in domain.iter() {
        let s = n.to_string();
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if chars.next().is_some() { panic!("Expected a single char in {}", s) }
        let point = point(
            ((pos[1] * cell_width + border_width + 1) as f32 + char_x as f32 * v_metrics.ascent),
            ((pos[0] + 1) * cell_width - 2) as f32 - char_y as f32 * v_metrics.ascent);
        let glyph = font.glyph(c).expect(&format!("No glyph for {}", c))
            .scaled(scale)
            .positioned(point);
        let bb = glyph.pixel_bounding_box().unwrap();
        glyph.draw(|x, y, v| {
            if v == 0.0 { return };
            let v = ((1.0 - v) * 255.0) as u8;
            image.put_pixel(
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
    image: &mut RgbImage,
    pos: Coord,
    value: i32,
    font: &Font,
    cell_width: usize,
    border_width: usize)
{
    let scale = Scale::uniform(cell_width as f32 * 0.8);
    let v_metrics = font.v_metrics(scale);

    let s = value.to_string();
    let mut chars = s.chars();
    let c = chars.next().unwrap();
    if let Some(c2) = chars.next() { panic!("Unexpected char: {}", c2) }
    let glyph = font.glyph(c).unwrap_or_else(|| panic!("No glyph for {}", c))
        .scaled(scale);
    let h_metrics = glyph.h_metrics();
    let x = (pos[1] * cell_width + border_width) as f32 + ((cell_width - border_width) as f32 - h_metrics.advance_width) / 2.0;
    let y = ((pos[0] + 1) * cell_width) as f32 - ((cell_width - border_width) as f32 - v_metrics.ascent) / 2.0;
    let point = point(x, y);
    println!("pos {} point {} {}, a {}, b {}", pos, x, y, cell_width - border_width, v_metrics.ascent);
    let glyph = glyph.positioned(point);
    let bb = glyph.pixel_bounding_box().unwrap();
    glyph.draw(|x, y, val| {
        if val == 0.0 { return };
        let val = ((1.0 - val) * 255.0) as u8;
        image.put_pixel(bb.min.x as u32 + x,
                        bb.min.y as u32 + y,
                        Rgb { data: [val; 3] });
    });
}
