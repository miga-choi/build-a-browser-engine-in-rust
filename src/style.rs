//! Code for applying CSS styles to the DOM.
//!
//! I will call it "CSS Renderer"

use crate::dom::{Element, Node};
use std::collections::HashMap;
use crate::css::{Rule, Selector, SimpleSelector, Specificity, Stylesheet, Value};
use crate::css::Selector::Simple;
/*
    The output of this engine's style module is something I call the "style tree".
    Each node in this tree includes a pointer to a DOM node, plus its CSS property values.
 */

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
    What’s with all the 'a stuff? Those are lifetimes, part of how Rust guarantees
    that pointers are memory-safe without requiring garbage collection. If you’re not
    working in Rust you can ignore them; they aren’t critical to the code’s meaning.

    e.g.
        StyledNode<'a> {
            node: &'a Node,
            specified_values: PropertyMap,
            children: Vec<StyledNode<'a>>,
        }
 */
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}


/// CSS's `display` enum
/*
    e.g.
        Display::Inline, Display::Block, Display::None
 */
pub enum Display {
    Inline,
    Block,
    None,
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

/// A single CSS rule and the specificity of its most specific matching selector.
type MatchedRule<'a> = (Specificity, &'a Rule);


/*
    The first step in building the style tree is [selector matching](https://www.w3.org/TR/CSS2/selector.html#pattern-matching).
    This will be very easy, since my CSS parser supports only simple selectors.
    You can tell whether a simple selector matches an element just by looking at
    the element itself. Matching compound selectors would require traversing
    the DOM tree to look at the element’s siblings, parents, etc.
 */

/// Selector matching:
fn matches(element: Element, selector: &Selector) -> bool {
    match selector {
        Simple(s) => matches_simple_selector(element, s)
    }
}

fn matches_simple_selector(element: Element, selector: &SimpleSelector) -> bool {
    // Check "tag" selector
    if selector.tag_name.iter().any(|name: &String| element.tag_name != *name) {
        return false;
    }

    // Check "id" selector
    if selector.id.iter().any(|id: &String| element.id() != Some(id)) {
        return false;
    }

    // Check "class" selectors
    if selector.class.iter().any(|class: &String| !element.classes().contains(class.as_str())) {
        return false;
    }

    // We didn't find any non-matching selector components.
    true
}