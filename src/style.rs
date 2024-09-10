//! Code for applying CSS styles to the DOM.

use crate::dom::Node;
use std::collections::HashMap;
use crate::css::Value;

/// Map from CSS property names to values.
/*
    e.g.
        PropertyMap {
            "background-color": Value::ColorValue(Color { r: 0, g: 0, b: 0, a: 1 })
        }
 }
 */
pub type PropertyMap = HashMap<String, Value>;


/// A node with associated style data.
/*
    e.g.
        StyledNode {
            node: Node,
            specified_values: PropertyMap,
            children: Vec<StyleNode<'a>>,
        }
 */
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}


/// CSS's `display` property
/*
    e.g.
        Display.Inline, Display.Block, Display.None
 */
pub enum Display {
    Inline, // display: inline
    Block,  // display: block
    None,   // display: none
}

impl<'a> StyledNode<'a> {
    /// Return the specified value of a property if it exists, otherwise `None`.
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    /// Return the specified value of property `name`, or property `fallback_name`
    /// if that doesn't exist, or value `default` if neither does.
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name).unwrap_or_else(
            || self.value(fallback_name).unwrap_or_else(|| default.clone())
        )
    }

    /// The value of the `display` property (defaults to inline).
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}