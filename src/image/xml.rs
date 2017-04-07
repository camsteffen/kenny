use std::fmt::{Display, Formatter, Result};

pub(crate) struct XmlProducer<F>(F)
where
    F: Fn(&mut Xml<'_, '_>) -> Result;

impl<F> XmlProducer<F>
where
    F: Fn(&mut Xml<'_, '_>) -> Result,
{
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<F> Display for XmlProducer<F>
where
    F: Fn(&mut Xml<'_, '_>) -> Result,
{
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        let mut xml = Xml::new(fmt);
        (self.0)(&mut xml)?;
        xml.finish()
    }
}

pub(crate) struct Xml<'a, 'b> {
    writer: &'a mut Formatter<'b>,
    elements: Vec<&'static str>,
    in_tag: bool,
}

impl<'a, 'b> Xml<'a, 'b> {
    pub fn new(writer: &'a mut Formatter<'b>) -> Self {
        Self {
            writer,
            elements: Vec::new(),
            in_tag: false,
        }
    }
}

impl Xml<'_, '_> {
    pub fn open_element(&mut self, name: &'static str) -> Result {
        if self.in_tag {
            writeln!(self.writer, ">")?;
        } else {
            self.in_tag = true;
        }
        self.elements.push(name);
        write!(self.writer, "<{}", name)
    }

    pub fn attribute(&mut self, name: &'static str, value: impl Display) -> Result {
        write!(self.writer, r#" {}="{}""#, name, value)
    }

    pub fn close_element(&mut self) -> Result {
        let name = self.elements.pop().unwrap();
        if self.in_tag {
            self.in_tag = false;
            writeln!(self.writer, "/>")
        } else {
            writeln!(self.writer, "</{}>", name)
        }
    }

    pub fn text(&mut self, text: impl Display) -> Result {
        if self.in_tag {
            write!(self.writer, ">")?;
            self.in_tag = false;
        }
        write!(self.writer, "{}", text)
    }

    pub fn finish(&mut self) -> Result {
        if self.in_tag {
            self.in_tag = false;
            writeln!(self.writer, "/>")?;
        }
        while let Some(name) = self.elements.pop() {
            writeln!(self.writer, "</{}>", name)?;
        }
        Ok(())
    }
}

macro_rules! xml {
    ($($xml:expr $(,)?)?) => {};

    ($xml:expr, open $name:literal $(, $($tail:tt)*)?) => {
        $xml.open_element($name)?;
        $(xml!($xml, $($tail)*))?
    };

    ($xml:expr, $arg_name:literal = $arg_value:expr $(, $($tail:tt)*)?) => {
        $xml.attribute($arg_name, $arg_value)?;
        $(xml!($xml, $($tail)*))?
    };

    ($xml:expr, text = $text:expr $(, $($tail:tt)*)?) => {
        $xml.text($text)?;
        $(xml!($xml, $($tail)*))?
    };

    ($xml:expr, close $(, $($tail:tt)*)?) => {
        $xml.close_element()?;
        $(xml!($xml, $($tail)*))?
    };

    ($xml:expr, if $($tail:tt)*) => {
        __xmlif!($xml, () ($($tail)*))
    };
}

#[doc(hidden)]
macro_rules! __xmlif {
    ($xml:expr, ($($cond:tt)*) ({ $($body:tt)* } $($tail:tt)*)) => {
        if $($cond)* {
            xml!($xml, $($body)*);
        }
        xml!($xml, $($tail)*);
    };

    ($xml:expr, ($($cond:tt)*) ($next:tt $($tail:tt)*)) => {
        __xmlif!($xml, ($($cond)* $next) ($($tail)*))
    };

    ($xml:expr, ($first:tt $($tail:tt)*) ()) => {
        compile_error!(concat!("expected `{`, found ", stringify!($first)))
    };
}
