//! Generate images for unsolved or solved puzzles

use crate::collections::square::{Coord, IsSquare, Square};
use crate::puzzle::solve::markup::{CellChange, CellChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::{Puzzle, Solution, Value};
use ahash::AHashSet;
use once_cell::sync::Lazy;
use std::ops::Deref;
use svg::node;
use svg::node::element::path::Data;
use svg::node::element::Path;
use svg::node::element::Rectangle;
use svg::node::element::Style;
use svg::node::element::Text;
use svg::node::Node;

const COLOR_CAGE_BORDER: &str = "black";
const COLOR_CELL_BORDER: &str = "#CDCDCD";
const COLOR_HIGHLIGHT: &str = "#FFFFC8";
const COLOR_DOMAIN_SLASH: &str = "red";

const CELL_WIDTH: i32 = 100;
const BORDER_WIDTH_CELL: i32 = 2;
const BORDER_WIDTH_CAGE: i32 = 4;
const BORDER_WIDTH_OUTER: i32 = 6;
const OUTER_PAD: i32 = BORDER_WIDTH_OUTER - BORDER_WIDTH_CELL / 2;
const DOMAIN_SLASH_WIDTH: &str = "1.4";

const CAGE_SPEC_FONT_SIZE: i32 = CELL_WIDTH / 4;
const CELL_SOLUTION_FONT_SIZE: i32 = 80;
const DOMAIN_FONT_SIZE: i32 = 20;
const DOMAIN_DX: i32 = 15;
const DOMAIN_PAD: i32 = 5;

const CAGE_SPEC_PAD: i32 = BORDER_WIDTH_CELL + CELL_WIDTH / 16;

static STYLE: Lazy<String> = Lazy::new(|| {
    format!(
        "text{{\
           font-family:sans-serif;\
         }}\
         .highlight{{\
           width:{cell_width}px;\
           height:{cell_width}px;\
           fill:{color_highlight};\
         }}\
         .cage-spec{{\
           font-size:{cage_spec_font_size}px;\
         }}\
         .solution{{\
           font-size:{solution_font_size}px;\
           text-anchor:middle;\
         }}\
         .domain{{\
           font-size:{domain_font_size}px;\
         }}\
         .domain-slash{{\
           stroke:{color_domain_slash};\
           stroke-width:{domain_slash_width};\
           stroke-linecap:round;\
         }}",
        cell_width = CELL_WIDTH,
        cage_spec_font_size = CAGE_SPEC_FONT_SIZE,
        domain_font_size = DOMAIN_FONT_SIZE,
        solution_font_size = CELL_SOLUTION_FONT_SIZE,
        color_highlight = COLOR_HIGHLIGHT,
        color_domain_slash = COLOR_DOMAIN_SLASH,
        domain_slash_width = DOMAIN_SLASH_WIDTH,
    )
});

/// Creates an image of a puzzle with optional markup
pub struct PuzzleImageBuilder<'a> {
    puzzle: &'a Puzzle,

    cell_variables: Option<&'a Square<CellVariable>>,
    solution: Option<&'a Solution>,
    cell_changes: Option<&'a CellChanges>,
}

impl<'a> PuzzleImageBuilder<'a> {
    pub fn new(puzzle: &'a Puzzle) -> Self {
        Self {
            puzzle,
            cell_variables: None,
            solution: None,
            cell_changes: None,
        }
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

    pub fn build(self) -> svg::Document {
        BuildContext::new(self).build()
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
    document: svg::Document,
    width: i32,
    cells_width: i32,
}

impl<'a> BuildContext<'a> {
    fn new(builder: PuzzleImageBuilder<'a>) -> Self {
        let cells_width = CELL_WIDTH * builder.puzzle.width() as i32;
        let width = cells_width + OUTER_PAD * 2;
        let document = svg::Document::new();
        Self {
            builder,
            document,
            width,
            cells_width,
        }
    }

    fn build(mut self) -> svg::Document {
        self.document = self.document.set("viewBox", (0, 0, self.width, self.width));
        self.document.append(Style::new(&*STYLE));
        self.background();
        self.highlight_cells();
        self.grid();
        self.cages_outline();
        self.outer_border();
        self.cage_spec();
        if let Some(cell_variables) = self.cell_variables {
            self.draw_markup(cell_variables);
        } else if let Some(solution) = self.solution {
            self.draw_solution(solution);
        }
        self.document
    }

    fn background(&mut self) {
        self.document.append(
            Rectangle::new()
                .set("x", OUTER_PAD)
                .set("y", OUTER_PAD)
                .set("width", self.cells_width)
                .set("height", self.cells_width)
                .set("fill", "white"),
        );
    }

    fn grid(&mut self) {
        let mut cell_data = Data::new();

        for i in 1..self.puzzle.width() {
            let coord = cell_coord(Coord::new(0, i));
            cell_data = cell_data
                .move_to(coord.as_tuple())
                .line_by((self.cells_width, 0))
                .move_to(coord.transpose().as_tuple())
                .line_by((0, self.cells_width));
        }

        self.document.append(
            Path::new()
                .set("stroke", COLOR_CELL_BORDER)
                .set("stroke-width", BORDER_WIDTH_CELL)
                .set("stroke-linecap", "round")
                .set("d", cell_data),
        );
    }

    fn cages_outline(&mut self) {
        let mut data = Data::new();
        for i in 0..self.puzzle.width() {
            for j in 1..self.puzzle.width() {
                // horizontal
                let pos1 = Coord::new(i, j - 1);
                let pos2 = Coord::new(i, j);
                if self.puzzle.cell(pos1).cage_id() != self.puzzle.cell(pos2).cage_id() {
                    data = data
                        .move_to(cell_coord(pos2).as_tuple())
                        .horizontal_line_by(CELL_WIDTH);
                }

                // vertical
                let pos1 = pos1.transpose();
                let pos2 = pos2.transpose();
                if self.puzzle.cell(pos1).cage_id() != self.puzzle.cell(pos2).cage_id() {
                    data = data
                        .move_to(cell_coord(pos2).as_tuple())
                        .vertical_line_by(CELL_WIDTH);
                }
            }
        }
        self.document.append(
            Path::new()
                .set("stroke", COLOR_CAGE_BORDER)
                .set("stroke-width", BORDER_WIDTH_CAGE)
                .set("stroke-linecap", "round")
                .set("d", data),
        );
    }

    fn outer_border(&mut self) {
        let x = BORDER_WIDTH_OUTER / 2;
        let width = self.width - BORDER_WIDTH_OUTER;
        self.document.append(
            Rectangle::new()
                .set("x", x)
                .set("y", x)
                .set("width", width)
                .set("height", width)
                .set("fill", "none")
                .set("stroke", COLOR_CAGE_BORDER)
                .set("stroke-width", BORDER_WIDTH_OUTER)
                .set("stroke-linejoin", "round"),
        );
    }

    fn cage_spec(&mut self) {
        for cage in self.puzzle.cages() {
            let text = match cage.operator().display_symbol() {
                Some(symbol) => format!("{}{}", cage.target(), symbol),
                None => cage.target().to_string(),
            };

            let pos = cell_coord(cage.coord());

            let text = Text::new()
                .add(node::Text::new(text))
                .set("class", "cage-spec")
                .set("x", pos.col() + CAGE_SPEC_PAD)
                .set("y", pos.row() + CAGE_SPEC_PAD)
                .set("dy", ".8em");
            self.document.append(text);
        }
    }

    fn highlight_cells(&mut self) {
        let cell_changes = match self.cell_changes {
            None => return,
            Some(cell_changes) => cell_changes,
        };
        for (&id, _) in cell_changes {
            let coord = cell_coord(self.puzzle.cell(id).coord());
            let rect = Rectangle::new()
                .set("class", "highlight")
                .set("x", coord.col())
                .set("y", coord.row());
            self.document.append(rect);
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
                        Some(&CellChange::Solution(value)) => {
                            domain.iter().filter(|&v| v != value).collect()
                        }
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
            let pos = cell_variables.cell(id).coord();
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
        const MAX_LINE_LEN: i32 = 5;

        if domain.len() as i32 > MAX_LINE_LEN * 2 {
            return;
        }
        let coord = cell_coord(pos);
        let mut char_x = 0;
        let mut char_y = 0;
        for n in domain {
            let s = n.to_string();
            let x = coord.col() + DOMAIN_PAD + char_x * DOMAIN_DX;
            let y = coord.row() + CELL_WIDTH - DOMAIN_PAD - char_y * DOMAIN_FONT_SIZE;
            self.document.append(
                Text::new()
                    .add(node::Text::new(s))
                    .set("class", "domain")
                    .set("x", x)
                    .set("y", y),
            );
            if removals.contains(&n) {
                self.document.append(
                    Path::new().set("class", "domain-slash").set(
                        "d",
                        Data::new()
                            .move_to((x, y))
                            .line_by((DOMAIN_FONT_SIZE / 2, -(DOMAIN_FONT_SIZE * 5 / 7))),
                    ),
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
        let coord = cell_coord(pos);
        self.document.append(
            Text::new()
                .add(node::Text::new(value.to_string()))
                .set("class", "solution")
                .set("x", coord.col() + CELL_WIDTH / 2)
                .set("y", coord.row() + CELL_WIDTH / 2)
                .set("dy", ".35em"),
        );
    }
}

fn cell_coord(coord: Coord<usize>) -> Coord<i32> {
    Coord::new(
        coord.col() as i32 * CELL_WIDTH + OUTER_PAD,
        coord.row() as i32 * CELL_WIDTH + OUTER_PAD,
    )
}
