//! Generate images for unsolved or solved puzzles

use std::borrow::Cow;
use std::fmt::{Result, Write};
use std::fs::File;
use std::io;
use std::io::{BufWriter, Write as ioWrite};
use std::path::Path;

use itertools::Itertools;
use once_cell::sync::Lazy;
use vec_map::VecMap;
use xml::Xml;

use crate::collections::square::{Coord, IsSquare, Square};
use crate::image::xml::XmlProducer;
use crate::puzzle::{CellId, Puzzle, Solution};
use crate::solve::markup::{CellChange, CellChanges};
use crate::solve::CellVariable;
use crate::solve::ValueSet;
use crate::HashSet;

#[macro_use]
mod xml;

// colors
const COLOR_CAGE_BORDER: &str = "black";
const COLOR_CELL_BORDER: &str = "#CCC";
const COLOR_HIGHLIGHT: &str = "#FFC";
const COLOR_DOMAIN: &str = "#444";
const COLOR_DOMAIN_SLASH: &str = "red";

// dimensions
const CELL_WIDTH: i32 = 100;
const BORDER_WIDTH_CELL: i32 = 2;
const BORDER_WIDTH_CAGE: i32 = 4;
const BORDER_WIDTH_OUTER: i32 = 6;
const OUTER_PAD: i32 = BORDER_WIDTH_OUTER - BORDER_WIDTH_CELL / 2;
const CAGE_SPEC_PAD: i32 = BORDER_WIDTH_CELL + CELL_WIDTH / 16;
const DOMAIN_SLASH_WIDTH: &str = "1.4";
const DOMAIN_PAD: i32 = 5;
const DOMAIN_DX: i32 = 15;
const MAX_DOMAIN_LINE_LEN: i32 = 5;

// font sizes
const FONT_SIZE_SOLUTION: i32 = 64;
const FONT_SIZE_CAGE_SPEC: i32 = 24;
const FONT_SIZE_DOMAIN: i32 = 20;

static STYLE: Lazy<String> = Lazy::new(|| {
    format!(
        "\
        text{{\
          font-family:sans-serif\
        }}\
        .highlight{{\
          fill:{color_highlight}\
        }}\
        .cage-spec{{\
          font-size:{cage_spec_font_size}px\
        }}\
        .solutions{{\
          font-size:{solution_font_size}px;\
          text-anchor:middle\
        }}\
        .new-solution{{\
          fill:green\
        }}\
        .domain{{\
          font-size:{domain_font_size}px;\
          fill:{color_domain}\
        }}",
        cage_spec_font_size = FONT_SIZE_CAGE_SPEC,
        domain_font_size = FONT_SIZE_DOMAIN,
        solution_font_size = FONT_SIZE_SOLUTION,
        color_domain = COLOR_DOMAIN,
        color_highlight = COLOR_HIGHLIGHT,
    )
});

/// Creates an image of a puzzle with optional markup
pub struct PuzzleImageBuilder<'a> {
    puzzle: &'a Puzzle,
    cell_changes: Option<&'a CellChanges>,
    cell_variables: Option<&'a Square<CellVariable>>,
    solution: Option<&'a Solution>,
}

impl<'a> PuzzleImageBuilder<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            cell_changes: None,
            solution: None,
            cell_variables: None,
        }
    }

    pub(crate) fn cell_changes(&mut self, cell_changes: &'a CellChanges) -> &mut Self {
        self.cell_changes = Some(cell_changes);
        self
    }

    pub(crate) fn cell_variables(
        &mut self,
        cell_variables: Option<&'a Square<CellVariable>>,
    ) -> &mut Self {
        self.cell_variables = cell_variables;
        self
    }

    pub fn solution(&mut self, solution: &'a Solution) -> &mut Self {
        self.solution = Some(solution);
        self
    }

    pub fn build(self) -> PuzzleImage<'a> {
        let puzzle = self.puzzle;
        let cells_width = CELL_WIDTH * puzzle.width() as i32;
        let width = cells_width + OUTER_PAD * 2;
        let (solutions, domains) = if let Some(cell_variables) = self.cell_variables {
            Self::solutions_domains(cell_variables, self.cell_changes)
        } else {
            let solutions = if let Some(solution) = self.solution {
                solution
                    .iter()
                    .enumerate()
                    .map(|(cell_id, &value)| SolutionValue {
                        cell_id,
                        value,
                        is_new: false,
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (solutions, VecMap::new())
        };
        let changed_cells = if let Some(cell_changes) = self.cell_changes {
            cell_changes.keys().copied().collect()
        } else {
            Vec::new()
        };
        PuzzleImage {
            puzzle,
            solutions,
            domains,
            changed_cells,
            cells_width,
            width,
        }
    }

    fn solutions_domains(
        cell_variables: &Square<CellVariable>,
        cell_changes: Option<&'a CellChanges>,
    ) -> (Vec<SolutionValue>, VecMap<Vec<DomainValue>>) {
        let mut solutions = Vec::new();
        let mut domains = VecMap::new();
        for (cell_id, cell) in cell_variables.iter().enumerate() {
            let cell_change = cell_changes.and_then(|cell_changes| cell_changes.get(cell_id));
            let domain_and_removals: Option<(&ValueSet, Cow<'_, HashSet<i32>>)> =
                if let CellVariable::Unsolved(ref domain) = cell {
                    let removals = if let Some(CellChange::DomainRemovals(ref values)) = cell_change
                    {
                        Cow::Borrowed(values)
                    } else {
                        Cow::Owned(HashSet::new())
                    };
                    Some((domain, removals))
                } else {
                    None
                };
            let solution_and_is_new = if let CellVariable::Solved(value) = *cell {
                Some((value, false))
            } else if let Some(&CellChange::Solution(value)) = cell_change {
                Some((value, true))
            } else {
                match domain_and_removals {
                    Some((ref domain, ref removals)) if domain.len() - removals.len() == 1 => {
                        // since there is one domain value left, show the solution
                        let value = domain.iter().find(|v| !removals.contains(v)).unwrap();
                        Some((value, true))
                    }
                    _ => None,
                }
            };
            if let Some((value, is_new)) = solution_and_is_new {
                solutions.push(SolutionValue {
                    cell_id,
                    value,
                    is_new,
                });
            }
            if let Some((domain, removals)) = domain_and_removals {
                let values = domain
                    .iter()
                    .map(|value| DomainValue {
                        value,
                        removed: removals.contains(&value),
                    })
                    .collect();
                domains.insert(cell_id, values);
            }
        }
        (solutions, domains)
    }
}

pub struct PuzzleImage<'a> {
    puzzle: &'a Puzzle,
    solutions: Vec<SolutionValue>,
    domains: VecMap<Vec<DomainValue>>,
    changed_cells: Vec<CellId>,
    width: i32,
    cells_width: i32,
}

struct DomainValue {
    value: i32,
    removed: bool,
}

struct SolutionValue {
    cell_id: usize,
    value: i32,
    is_new: bool,
}

impl<'x> PuzzleImage<'x> {
    pub fn save_svg(&self, path: &Path) -> io::Result<()> {
        let xml = XmlProducer::new(|xml| PuzzleSvgContext { image: self, xml }.write());
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        write!(writer, "{}", xml)?;
        writer.flush()?;
        Ok(())
    }
}

struct PuzzleSvgContext<'a, 'b, 'c> {
    image: &'a PuzzleImage<'a>,
    xml: &'a mut Xml<'b, 'c>,
}

impl PuzzleSvgContext<'_, '_, '_> {
    fn write(mut self) -> Result {
        self.header()?;
        self.background()?;
        self.highlight_cells()?;
        self.grid()?;
        self.outer_border()?;
        self.cages_outline()?;
        self.cage_spec()?;
        self.domain()?;
        self.solutions()
    }

    fn header(&mut self) -> Result {
        xml! {
            self.xml,
            open "svg",
            "xmlns" = "http://www.w3.org/2000/svg",
            "viewBox" = format!("0 0 {0} {0}", self.image.width),
            open "style",
            text = STYLE.as_str(),
            close,
        }
        Ok(())
    }

    fn background(&mut self) -> Result {
        xml! {
            self.xml,
            open "rect",
            "x" = OUTER_PAD,
            "y" = OUTER_PAD,
            "width" = self.image.cells_width,
            "height" = self.image.cells_width,
            "fill" = "white",
            close,
        }
        Ok(())
    }

    fn highlight_cells(&mut self) -> Result {
        if self.image.changed_cells.is_empty() {
            return Ok(());
        }
        xml!(self.xml, open "g", "class" = "highlight");
        for &cell_id in &self.image.changed_cells {
            let coord = self.cell_id_coord(cell_id);
            xml! {
                self.xml,
                open "rect",
                "x" = coord.col(),
                "y" = coord.row(),
                "width" = CELL_WIDTH,
                "height" = CELL_WIDTH,
                close,
            }
        }
        xml!(self.xml, close);
        Ok(())
    }

    fn grid(&mut self) -> Result {
        let mut d = String::new();
        for i in 1..self.image.puzzle.width() {
            let coord = cell_coord(Coord::new(0, i));
            write!(
                &mut d,
                "M{0}h{2}M{1}v{2}",
                path_coord(coord),
                path_coord(coord.transpose()),
                self.image.cells_width,
            )
            .unwrap();
        }
        xml! {
            self.xml,
            open "path",
            "stroke" = COLOR_CELL_BORDER,
            "stroke-width" = BORDER_WIDTH_CELL,
            "d" = d,
            close,
        }
        Ok(())
    }

    fn outer_border(&mut self) -> Result {
        let x = BORDER_WIDTH_OUTER / 2;
        let width = self.image.width - BORDER_WIDTH_OUTER;
        xml! {
            self.xml,
            open "rect",
            "x" = x,
            "y" = x,
            "width" = width,
            "height" = width,
            "fill" = "none",
            "stroke" = COLOR_CAGE_BORDER,
            "stroke-width" = BORDER_WIDTH_OUTER,
            "stroke-linejoin" = "round",
            close,
        }
        Ok(())
    }

    fn cages_outline(&mut self) -> Result {
        struct Direction {
            coord_a: fn(usize, usize) -> Coord,
            coord_b: fn(usize, usize) -> Coord,
            draw_char: char,
        }
        const DIRECTIONS: [Direction; 2] = [
            // draw vertical lines between left-right adjacent cells
            Direction {
                coord_a: |a, b| Coord::new(a, b),
                coord_b: |a, b| Coord::new(a + 1, b),
                draw_char: 'v',
            },
            // draw horizontal lines between top-bottom adjacent cells
            Direction {
                coord_a: |a, b| Coord::new(b, a),
                coord_b: |a, b| Coord::new(b, a + 1),
                draw_char: 'h',
            },
        ];

        let puzzle = self.image.puzzle;
        let mut d = String::new();
        for direction in &DIRECTIONS {
            let Direction {
                coord_a,
                coord_b,
                draw_char,
            } = direction;
            for i in 0..puzzle.width() - 1 {
                let lines = (0..puzzle.width())
                    // if the two adjacent cells have different cages, draw a line
                    .filter(|&j| {
                        let (a, b) = (coord_a(i, j), coord_b(i, j));
                        let (cell_a, cell_b) = (puzzle.cell(a), puzzle.cell(b));
                        cell_a.cage_id() != cell_b.cage_id()
                    })
                    // line start position and line length in cells
                    .map(|j| (j, 1))
                    // combine connected line segments
                    .coalesce(|a, b| {
                        if b.0 == a.0 + a.1 {
                            Ok((a.0, a.1 + b.1))
                        } else {
                            Err((a, b))
                        }
                    });
                for (j, len) in lines {
                    write!(
                        d,
                        "M{}{}{}",
                        // the second cell's coordinates is where the line starts
                        path_coord(cell_coord(coord_b(i, j))),
                        // horizontal or vertical
                        draw_char,
                        CELL_WIDTH * len as i32
                    )
                    .unwrap();
                }
            }
        }
        xml! {
            self.xml,
            open "path",
            "stroke" = COLOR_CAGE_BORDER,
            "stroke-width" = BORDER_WIDTH_CAGE,
            "stroke-linecap" = "round",
            "d" = d,
            close,
        }
        Ok(())
    }

    fn cage_spec(&mut self) -> Result {
        xml!(self.xml, open "g", "class" = "cage-spec");
        for cage in self.image.puzzle.cages() {
            let text = match cage.operator().display_symbol() {
                Some(symbol) => format!("{}{}", cage.target(), symbol),
                None => cage.target().to_string(),
            };
            let pos = cell_coord(cage.coord());
            xml!(
                self.xml,
                open "text",
                "x" = pos.col() + CAGE_SPEC_PAD,
                "y" = pos.row() + CAGE_SPEC_PAD,
                "dy" = ".8em",
                text = text,
                close,
            );
        }
        xml!(self.xml, close);
        Ok(())
    }

    fn domain(&mut self) -> Result {
        if self.image.domains.is_empty() {
            return Ok(());
        }

        xml!(self.xml, open "g", "class" = "domain");

        let mut removals = Vec::new();

        for (cell_id, domain) in &self.image.domains {
            if domain.len() as i32 > MAX_DOMAIN_LINE_LEN * 2 {
                // domain is too long to show
                continue;
            }
            let coord = self.cell_id_coord(cell_id);
            let mut char_x = 0;
            let mut char_y = 0;
            for &DomainValue { value, removed } in domain {
                let x = coord.col() + DOMAIN_PAD + char_x * DOMAIN_DX;
                let y = coord.row() + CELL_WIDTH - DOMAIN_PAD - char_y * FONT_SIZE_DOMAIN;
                xml! {
                    self.xml,
                    open "text",
                    "x" = x,
                    "y" = y,
                    text = value,
                    close,
                }
                if removed {
                    removals.push((x, y));
                }
                char_x += 1;
                if char_x == MAX_DOMAIN_LINE_LEN {
                    char_x = 0;
                    char_y += 1;
                }
            }
        }
        xml!(self.xml, close);
        if !removals.is_empty() {
            let mut d = String::new();
            for (x, y) in removals {
                write!(
                    d,
                    "M{},{}l{},{}",
                    x,
                    y,
                    FONT_SIZE_DOMAIN / 2,
                    -(FONT_SIZE_DOMAIN * 5 / 7),
                )
                .unwrap();
            }
            xml! {
                self.xml,
                open "path",
                "stroke" = COLOR_DOMAIN_SLASH,
                "stroke-width" = DOMAIN_SLASH_WIDTH,
                "stroke-linecap" = "round",
                "d" = d,
                close,
            }
        }
        Ok(())
    }

    fn solutions(&mut self) -> Result {
        if self.image.solutions.is_empty() {
            return Ok(());
        }
        xml!(self.xml, open "g", "class" = "solutions");
        for solution_value in &self.image.solutions {
            let &SolutionValue {
                cell_id,
                value,
                is_new,
            } = solution_value;
            let coord = self.cell_id_coord(cell_id);
            xml! {
                self.xml,
                open "text",
                if is_new {
                    "class" = "new-solution",
                }
                "x" = coord.col() + CELL_WIDTH / 2,
                "y" = coord.row() + CELL_WIDTH / 2,
                "dy" = ".35em",
                text = value,
                close,
            }
        }
        xml!(self.xml, close);
        Ok(())
    }

    fn cell_id_coord(&self, cell_id: CellId) -> Coord<i32> {
        cell_coord(self.image.puzzle.cell(cell_id).coord())
    }
}

fn cell_coord(coord: Coord<usize>) -> Coord<i32> {
    Coord::new(
        coord.col() as i32 * CELL_WIDTH + OUTER_PAD,
        coord.row() as i32 * CELL_WIDTH + OUTER_PAD,
    )
}

fn path_coord(coord: Coord<i32>) -> String {
    format!("{},{}", coord.col(), coord.row())
}
