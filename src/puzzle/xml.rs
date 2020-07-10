use std::fmt::Display;
use std::io;
use std::io::Write;

macro_rules! xml {
    ($xml:expr, $name:literal $(, $arg_name:literal = $arg_value:expr)*,open $(,)?) => {
        $xml.element($name)
        $(.and_then(|_| $xml.attr($arg_name, $arg_value)))*
    };

    ($xml:expr, $name:literal $(,$arg_name:literal = $arg_value:expr)* $(, text: $text:expr)? $(,)?) => {
        $xml.element($name)
        $(.and_then(|_| $xml.attr($arg_name, $arg_value)))*
        $(.and_then(|_| $xml.text($text)))*
        .and_then(|_| $xml.close_element())
    };
}

pub struct Xml<'a, W>
where
    W: Write,
{
    writer: &'a mut W,
    elements: Vec<&'static str>,
    in_tag: bool,
}

impl<'a, W> Xml<'a, W>
where
    W: Write,
{
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            elements: Vec::new(),
            in_tag: false,
        }
    }
}

impl<W> Xml<'_, W>
where
    W: Write,
{
    pub fn element(&mut self, name: &'static str) -> io::Result<()> {
        if self.in_tag {
            writeln!(self.writer, ">")?;
        } else {
            self.in_tag = true;
        }
        self.elements.push(name);
        write!(self.writer, "<{}", name)
    }

    pub fn attr(&mut self, name: &'static str, value: impl Display) -> io::Result<()> {
        write!(self.writer, " {}=\"{}\"", name, value)
    }

    pub fn close_element(&mut self) -> io::Result<()> {
        let name = self.elements.pop().unwrap();
        if self.in_tag {
            writeln!(self.writer, "/>")?;
        } else {
            writeln!(self.writer, "</{}>", name)?;
        }
        self.in_tag = false;
        Ok(())
    }

    pub fn text(&mut self, text: impl Display) -> io::Result<()> {
        if self.in_tag {
            write!(self.writer, ">")?;
            self.in_tag = false;
        }
        write!(self.writer, "{}", text)
    }
}
