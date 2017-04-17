extern crate png;
extern crate image;

use rusttype::Font;
use rusttype::FontCollection;
use rusttype::point;
use rusttype::Scale;
use square::*;
use board::*;
use solve::*;
use image::RgbImage;
use image::Rgb;

const BLACK: Rgb<u8> = Rgb { data: [0; 3] };
const WHITE: Rgb<u8> = Rgb { data: [255; 3] };

const COLOR_CELL_BORDER:  Rgb<u8> = Rgb { data: [205; 3] };
const COLOR_CAGE_BORDER: Rgb<u8> = BLACK;
const COLOR_BG: Rgb<u8> = WHITE;

pub fn image(cages: &[Cage], markup: &BoardMarkup, size: usize) {
    let cell_width = 60 as usize;
    let border_width = cell_width / 25;

    let image_width = (cell_width * size + border_width) as u32;
    let mut image = RgbImage::from_pixel(image_width, image_width, COLOR_BG);

    // draw grid
    draw_grid(&mut image, size, cell_width as u32, border_width as u32, cages);
    draw_cage_glyphs(&mut image, cages, markup, size, cell_width, border_width);

    image.save("image.png");
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
    size: usize,
    cell_width: u32,
    border_width: u32,
    cages: &[Cage])
{
    let image_width = cell_width * size as u32 + border_width;
    let cells_width = cell_width * size as u32;

    // draw outer border
    draw_rectangle(image, 0, 0, cells_width, border_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, cells_width, 0, image_width, cells_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, border_width, cells_width, image_width, image_width, COLOR_CAGE_BORDER);
    draw_rectangle(image, 0, border_width, border_width, image_width, COLOR_CAGE_BORDER);

    let cage_map = cage_map(cages, size);

    // draw horizontal line segments
    for i in 1..size { // row
        for j in 0..size { // col
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
    for i in 0..size { // row
        for j in 1..size { // col
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
    for i in 1..size {
        for j in 1..size {
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
    markup: &BoardMarkup,
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
        let operator_symbol = operator_symbol(&cage.operator);
        let text = &format!("{}{}", operator_symbol, cage.target);

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
                image.put_pixel(bb.min.x as u32 + x, bb.min.y as u32 + y, Rgb { data: [v; 3] });
            });
        }
    }

    // markup candidates
    for (pos, cell) in markup.cells.iter() {
        let candidates = match cell {
            &Unknown::Unsolved(ref candidates) => draw_cell_candidates(image, pos, candidates, &font, cell_width, border_width),
            &Unknown::Solved(value) => draw_cell_solution(image, pos, value, &font, cell_width, border_width),
        };
    }
}

fn draw_cell_candidates(
    image: &mut RgbImage,
    pos: Coord,
    candidates: &Candidates,
    font: &Font,
    cell_width: usize,
    border_width: usize)
{
    const max_line_len: usize = 5;

    let scale = Scale::uniform(cell_width as f32 * 0.2);
    let v_metrics = font.v_metrics(scale);

    if candidates.count > max_line_len * 2 { return }
    let num_lines = (candidates.count - 1) / max_line_len + 1;
    let mut char_x = 0;
    let mut char_y = 0;
    for candidate in candidates.iter_candidates() {
        let s = candidate.to_string();
        let mut chars = s.chars();
        let c = chars.next().unwrap();
        if let Some(c2) = chars.next() { panic!("Unexpected candidate char: {}", c2) }
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
            image.put_pixel(bb.min.x as u32 + x, bb.min.y as u32 + y, Rgb { data: [v; 3] });
        });
        char_x += 1;
        if char_x == max_line_len {
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
    let glyph = font.glyph(c).expect(&format!("No glyph for {}", c))
        .scaled(scale);
    let h_metrics = glyph.h_metrics();
    let x = (pos[1] * cell_width + border_width) as f32 + ((cell_width - border_width) as f32 - h_metrics.advance_width) / 2.0;
    let y = ((pos[0] + 1) * cell_width) as f32 - ((cell_width - border_width) as f32 - v_metrics.ascent) / 2.0;
    let point = point(x, y);
    println!("pos {} point {} {}, a {}, b {}", pos, x, y, cell_width - border_width, v_metrics.ascent);
    let glyph = glyph.positioned(point);
    let bb = glyph.pixel_bounding_box().unwrap();
    glyph.draw(|x, y, v| {
        if v == 0.0 { return };
        let v = ((1.0 - v) * 255.0) as u8;
        image.put_pixel(bb.min.x as u32 + x, bb.min.y as u32 + y, Rgb { data: [v; 3] });
    });
}
