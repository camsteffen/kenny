//! Generate images for unsolved or solved puzzles

use crate::collections::square::{Coord, IsSquare, Square};
use crate::puzzle::solve::markup::{CellChange, CellChanges};
use crate::puzzle::solve::CellVariable;
use crate::puzzle::solve::ValueSet;
use crate::puzzle::{Puzzle, Solution, Value};
use ahash::AHashSet;
use once_cell::sync::Lazy;
use std::fmt::{Display, Formatter, Write as fmtWrite};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{fmt, io};
use xml::Xml;

type Result = io::Result<()>;

macro_rules! xml_element {
    ($xml:expr, $name:literal $(, $arg_name:literal = $arg_value:expr)* $(, text $text:expr)? $(,)?) => {
        $xml.element($name)
        $(.and_then(|_| $xml.attr($arg_name, $arg_value)))*
        $(.and_then(|_| $xml.text($text)))*
        .and_then(|_| $xml.close_element())
    }
}

// colors
const COLOR_CAGE_BORDER: &str = "black";
const COLOR_CELL_BORDER: &str = "#CDCDCD";
const COLOR_HIGHLIGHT: &str = "#FFFFC8";
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

// font sizes
const FONT_SIZE_SOLUTION: i32 = 80;
const FONT_SIZE_CAGE_SPEC: i32 = 25;
const FONT_SIZE_DOMAIN: i32 = 20;

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
         .new-solution{{\
           fill:green;\
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
        cage_spec_font_size = FONT_SIZE_CAGE_SPEC,
        domain_font_size = FONT_SIZE_DOMAIN,
        solution_font_size = FONT_SIZE_SOLUTION,
        color_highlight = COLOR_HIGHLIGHT,
        color_domain_slash = COLOR_DOMAIN_SLASH,
        domain_slash_width = DOMAIN_SLASH_WIDTH,
    )
});

/// Creates an image of a puzzle with optional markup
pub struct PuzzleImage<'a> {
    puzzle: &'a Puzzle,
    cell_changes: Option<&'a CellChanges>,
    cell_variables: Option<&'a Square<CellVariable>>,
    solution: Option<&'a Solution>,
}

impl<'a> PuzzleImage<'a> {
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

    pub fn save(&self, path: &Path) -> io::Result<()> {
        self.write(&mut BufWriter::new(File::create(path)?))
    }

    pub fn to_string(&self) -> String {
        let mut bytes = Vec::new();
        self.write(&mut bytes).unwrap();
        unsafe { String::from_utf8_unchecked(bytes) }
    }

    pub fn write(&self, writer: &mut impl Write) -> Result {
        WriteContext::new(self, writer).write()
    }
}

struct WriteContext<'a, W: Write> {
    xml: Xml<'a, W>,
    width: i32,
    cells_width: i32,
    puzzle: &'a Puzzle,
    cell_changes: Option<&'a CellChanges>,
    cell_variables: Option<&'a Square<CellVariable>>,
    solution: Option<&'a Solution>,
}

impl<'a, W> WriteContext<'a, W>
where
    W: Write,
{
    fn new(image: &'a PuzzleImage<'_>, writer: &'a mut W) -> Self {
        let cells_width = CELL_WIDTH * image.puzzle.width() as i32;
        let width = cells_width + OUTER_PAD * 2;
        let xml = Xml::new(writer);
        Self {
            xml,
            width,
            cells_width,
            puzzle: image.puzzle,
            cell_changes: image.cell_changes,
            cell_variables: image.cell_variables,
            solution: image.solution,
        }
    }

    fn write(mut self) -> Result {
        self.xml.start()?;
        self.xml.element("svg")?;
        self.xml.attr("xmlns", "http://www.w3.org/2000/svg")?;
        self.xml
            .attr("viewBox", format!("0 0 {0} {0}", self.width))?;
        xml_element!(self.xml, "style", "type" = "text/css", text & *STYLE,)?;
        self.background()?;
        self.highlight_cells()?;
        self.grid()?;
        self.cages_outline()?;
        self.outer_border()?;
        self.cage_spec()?;
        if let Some(cell_variables) = self.cell_variables {
            self.markup(cell_variables)?;
        } else if let Some(solution) = self.solution {
            self.solution(solution)?;
        }
        self.xml.close_element()?;
        Ok(())
    }

    fn background(&mut self) -> io::Result<()> {
        xml_element!(
            self.xml,
            "rect",
            "x" = OUTER_PAD,
            "y" = OUTER_PAD,
            "width" = self.cells_width,
            "height" = self.cells_width,
            "fill" = "white",
        )
    }

    fn grid(&mut self) -> io::Result<()> {
        let mut cell_data = String::new();

        for i in 1..self.puzzle.width() {
            let coord = cell_coord(Coord::new(0, i));
            write!(
                &mut cell_data,
                "M {0} h {2} M {1} v {2} ",
                coord.as_array().lazy_join(','),
                coord.transpose().as_array().lazy_join(','),
                self.cells_width,
            )
            .unwrap();
        }

        xml_element!(
            self.xml,
            "path",
            "stroke" = COLOR_CELL_BORDER,
            "stroke-width" = BORDER_WIDTH_CELL,
            "d" = cell_data,
        )
    }

    fn cages_outline(&mut self) -> io::Result<()> {
        let mut data = String::new();
        for i in 0..self.puzzle.width() {
            for j in 1..self.puzzle.width() {
                // horizontal
                let pos1 = Coord::new(i, j - 1);
                let pos2 = Coord::new(i, j);
                if self.puzzle.cell(pos1).cage_id() != self.puzzle.cell(pos2).cage_id() {
                    write!(
                        data,
                        "M {} h {} ",
                        cell_coord(pos2).as_array().lazy_join(','),
                        CELL_WIDTH
                    )
                    .unwrap();
                }

                // vertical
                let pos1 = pos1.transpose();
                let pos2 = pos2.transpose();
                if self.puzzle.cell(pos1).cage_id() != self.puzzle.cell(pos2).cage_id() {
                    write!(
                        data,
                        "M {} h {} ",
                        cell_coord(pos2).as_array().lazy_join(','),
                        CELL_WIDTH
                    )
                    .unwrap();
                }
            }
        }
        xml_element!(
            self.xml,
            "path",
            "stroke" = COLOR_CAGE_BORDER,
            "stroke-width" = BORDER_WIDTH_CAGE,
            "stroke-linecap" = "round",
            "d" = data,
        )
    }

    fn outer_border(&mut self) -> io::Result<()> {
        let x = BORDER_WIDTH_OUTER / 2;
        let width = self.width - BORDER_WIDTH_OUTER;
        xml_element!(
            self.xml,
            "rect",
            "x" = x,
            "y" = x,
            "width" = width,
            "height" = width,
            "fill" = "none",
            "stroke" = COLOR_CAGE_BORDER,
            "stroke-width" = BORDER_WIDTH_OUTER,
            "stroke-linejoin" = "round",
        )
    }

    fn cage_spec(&mut self) -> io::Result<()> {
        for cage in self.puzzle.cages() {
            let text = match cage.operator().display_symbol() {
                Some(symbol) => format!("{}{}", cage.target(), symbol),
                None => cage.target().to_string(),
            };

            let pos = cell_coord(cage.coord());

            xml_element!(
                self.xml,
                "text",
                "x" = pos.col() + CAGE_SPEC_PAD,
                "y" = pos.col() + CAGE_SPEC_PAD,
                "dy" = ".8em",
                text text,
            )?;
        }
        Ok(())
    }

    fn highlight_cells(&mut self) -> io::Result<()> {
        let cell_changes = match self.cell_changes {
            None => return Ok(()),
            Some(cell_changes) => cell_changes,
        };
        for (&id, _) in cell_changes {
            let coord = cell_coord(self.puzzle.cell(id).coord());
            xml_element!(
                self.xml,
                "rect",
                "class" = "highlight",
                "x" = coord.col(),
                "y" = coord.row(),
            )?;
        }
        Ok(())
    }

    fn markup(&mut self, cell_variables: &Square<CellVariable>) -> io::Result<()> {
        for (id, cell) in cell_variables.iter().enumerate() {
            let cell_change = self
                .cell_changes
                .and_then(|cell_changes| cell_changes.get(id));
            let domain_and_removals: Option<(&ValueSet, AHashSet<i32>)> =
                if let CellVariable::Unsolved(ref domain) = cell {
                    let removals = if let Some(CellChange::DomainRemovals(ref values)) = cell_change
                    {
                        values.iter().copied().collect()
                    } else {
                        AHashSet::default()
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
            let pos = cell_variables.cell(id).coord();
            if let Some((value, is_new)) = solution_and_is_new {
                self.cell_solution(pos, value, is_new)?;
            }
            if let Some((domain, removals)) = domain_and_removals {
                self.domain(pos, &domain, &removals)?;
            }
        }
        Ok(())
    }

    fn solution(&mut self, solution: &Square<Value>) -> io::Result<()> {
        for (pos, value) in solution.iter_coord() {
            self.cell_solution(pos, *value, false)?;
        }
        Ok(())
    }

    fn domain(
        &mut self,
        pos: Coord,
        domain: &ValueSet,
        removals: &AHashSet<i32>,
    ) -> io::Result<()> {
        // the maximum number of characters that can fit on one line in a cell
        const MAX_LINE_LEN: i32 = 5;

        if domain.len() as i32 > MAX_LINE_LEN * 2 {
            return Ok(());
        }
        let coord = cell_coord(pos);
        let mut char_x = 0;
        let mut char_y = 0;
        for n in domain {
            let x = coord.col() + DOMAIN_PAD + char_x * DOMAIN_DX;
            let y = coord.row() + CELL_WIDTH - DOMAIN_PAD - char_y * FONT_SIZE_DOMAIN;
            xml_element!(
                self.xml,
                "text",
                "x" = x,
                "y" = y,
                text n,
            )?;
            if removals.contains(&n) {
                xml_element!(
                    self.xml,
                    "path",
                    "class" = "domain-slash",
                    "d" = format!(
                        "M {},{} l {},{}",
                        x,
                        y,
                        FONT_SIZE_DOMAIN / 2,
                        -(FONT_SIZE_DOMAIN * 5 / 7)
                    ),
                )?;
            }
            char_x += 1;
            if char_x == MAX_LINE_LEN {
                char_x = 0;
                char_y += 1;
            }
        }
        Ok(())
    }

    fn cell_solution(&mut self, pos: Coord, value: i32, is_new: bool) -> io::Result<()> {
        let coord = cell_coord(pos);
        xml_element!(
            self.xml,
            "text",
            "class" = format!("solution{}", if is_new { " new-solution" } else { "" }),
            "x" = coord.col() + CELL_WIDTH / 2,
            "y" = coord.row() + CELL_WIDTH / 2,
            "dy" = ".35em",
            text value,
        )
    }
}

fn cell_coord(coord: Coord<usize>) -> Coord<i32> {
    Coord::new(
        coord.col() as i32 * CELL_WIDTH + OUTER_PAD,
        coord.row() as i32 * CELL_WIDTH + OUTER_PAD,
    )
}

mod xml {
    use std::fmt::Display;
    use std::io;
    use std::io::Write;

    #[derive(PartialEq, PartialOrd)]
    enum State {
        Init,
        InTag,
        AfterChild,
    }

    pub struct Xml<'a, W>
    where
        W: Write,
    {
        writer: &'a mut W,
        elements: Vec<&'static str>,
        state: State,
    }

    impl<'a, W> Xml<'a, W>
    where
        W: Write,
    {
        pub fn new(writer: &'a mut W) -> Self {
            Self {
                writer,
                elements: Vec::new(),
                state: State::Init,
            }
        }
    }

    impl<W> Xml<'_, W>
    where
        W: Write,
    {
        pub fn start(&mut self) -> io::Result<()> {
            write!(
                self.writer,
                r#"<?xml version="1.0" standalone="yes"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN"
  "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
"#
            )
        }

        pub fn element(&mut self, name: &'static str) -> io::Result<()> {
            self.node_border()?;
            self.state = State::InTag;
            self.elements.push(name);
            write!(self.writer, "<{}", name)
        }

        pub fn attr(&mut self, name: &'static str, value: impl Display) -> io::Result<()> {
            write!(self.writer, " {}=\"{}\"", name, value)
        }

        pub fn close_element(&mut self) -> io::Result<()> {
            let in_tag = self.state == State::InTag;
            let name = self.elements.pop().unwrap();
            self.state = State::Init;
            if in_tag {
                write!(self.writer, "/>")
            } else {
                write!(self.writer, "</{}>", name)
            }
        }

        pub fn text(&mut self, text: impl Display) -> io::Result<()> {
            self.node_border()?;
            self.state = State::Init;
            write!(self.writer, "{}", text)
        }

        fn node_border(&mut self) -> io::Result<()> {
            if self.state == State::InTag {
                write!(self.writer, ">")?;
            }
            Ok(())
        }
    }
}

trait LazyJoinExt: Copy + IntoIterator + Sized
where
    Self::Item: Display,
{
    fn lazy_join<S>(self, separator: S) -> LazyJoin<Self, S>
    where
        S: Display;
}

struct LazyJoin<T, U>
where
    T: Copy + IntoIterator,
    T::Item: Display,
    U: Display,
{
    elements: T,
    separator: U,
}

impl<T> LazyJoinExt for T
where
    T: Copy + IntoIterator,
    T::Item: Display,
{
    fn lazy_join<S>(self, separator: S) -> LazyJoin<Self, S>
    where
        S: Display,
    {
        LazyJoin {
            elements: self,
            separator,
        }
    }
}

impl<T, S> Display for LazyJoin<T, S>
where
    T: Copy + IntoIterator,
    T::Item: Display,
    S: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut iter = self.elements.into_iter();
        let e = match iter.next() {
            None => return Ok(()),
            Some(e) => e,
        };
        write!(f, "{}", e)?;
        while let Some(e) = iter.next() {
            write!(f, "{}{}", self.separator, e)?;
        }
        Ok(())
    }
}
