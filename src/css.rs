/*
    A CSS stylesheet is a series of rules.
 */
use std::collections::btree_map::Values;

struct Stylesheet {
    rules: Vec<Rule>,
}


/*
    A rule includes one or more selectors separated by commas,
    followed by a series of declarations enclosed in braces.
 */

struct Rule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}


/*
    A selector can be a simple selector, or it can be a chain of selectors joined by combinators.
    This engine supports only simple selectors for now.

    In this engine, a simple selector can include a tag name, an ID prefixed by '#',
    any number of class names prefixed by '.', or some combination of the above.
    If the tag name is empty or '*' then it is a "universal selector" that can match any tag.

    There are many other types of selector (especially in CSS3), but this will do for now.
 */

enum Selector {
    Simple(SimpleSelector),
}

struct SimpleSelector {
    tag_name: Option<String>,
    id: Option<String>,
    class: Vec<String>,
}


/*
    A declaration is just a name/value pair, separated by a colon and ending with a semicolon.
 */

struct Declaration {
    name: String,
    value: Value,
}


/*
    This engine supports only a handful of CSS's many value types.
 */

enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values here
}

enum Unit {
    Px,
    // insert more values here
}


/*
    Rust note: u8 is an 8-bit unsigned integer, and f32 is a 32-bit float.
 */

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

