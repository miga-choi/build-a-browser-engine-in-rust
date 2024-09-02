/*
    A CSS stylesheet is a series of rules.
 */
use std::collections::btree_map::Values;
use std::num::ParseIntError;
use std::panic::resume_unwind;

pub struct Stylesheet {
    pub rules: Vec<Rule>,
}


/*
    A rule includes one or more selectors separated by commas,
    followed by a series of declarations enclosed in braces.
 */

pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}


/*
    A selector can be a [simple selector](https://www.w3.org/TR/CSS2/selector.html#selector-syntax),
    or it can be a chain of selectors joined by combinators.
    This engine supports only simple selectors for now.

    In this engine, a simple selector can include a tag name, an ID prefixed by '#',
    any number of class names prefixed by '.', or some combination of the above.
    If the tag name is empty or '*' then it is a "universal selector" that can match any tag.

    There are many other types of selector (especially in CSS3), but this will do for now.
 */

pub enum Selector {
    Simple(SimpleSelector),
}

pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}


/*
    A declaration is just a name/value pair, separated by a colon and ending with a semicolon.
 */

pub struct Declaration {
    pub name: String,
    pub value: Value,
}


/*
    This engine supports only a handful of CSS's many value types.
 */

pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values here
}

pub enum Unit {
    Px,
    // insert more values here
}


/*
    Rust note: u8 is an 8-bit unsigned integer, and f32 is a 32-bit float.
 */

pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}


/*
    CSS has a straightforward [grammar](https://www.w3.org/TR/CSS2/grammar.html),
    making it easier to parse correctly than its quirky cousin HTML.
    When a standards-compliant CSS parser encounters a [parse error](https://www.w3.org/TR/CSS2/syndata.html#parsing-errors),
    it discards the unrecognized part of the stylesheet but still process the remaining portions.
    This is useful because it allows stylesheets to include new syntax but still produce well-defined
    output in older browsers.
 */

struct Parser {
    pos: usize,
    input: String,
}


/// Parse a whole CSS stylesheet.
pub fn parse(source: String) -> Stylesheet {
    let mut parser = Parser { pos: 0, input: source };
    Stylesheet { rules: parser.parse_rules() }
}

impl Parser {
    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }
}


fn valid_identifier_char(c: char) -> bool {
    // TODO: Include U+00A0 and higher.
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}
