use std::collections::HashMap;
use crate::css::{Stylesheet, Rule, Selector, SimpleSelector, Value, Specificity};
use crate::dom::{Node, NodeType, ElementData, elem};

/*
    The Style Tree

    The output of this engine's style module is something I call the style tree.
    Each node in this tree includes a pointer to a DOM node, plus its CSS property values.
 */

/// Map from CSS property names to values.
type PropertyMap = HashMap<String, Value>;

/*
    What's with all the 'a stuff? Those are [lifetimes](https://doc.rust-lang.org/book/ownership.html),
    part of how Rust guarantees that pointers are memory-safe without requiring garbage collection.
    If you're not working in Rust you can ignore them; they aren't critical to the code's meaning.

    We could add new fields to the dom::Node struct instead of creating a new tree, but I wanted to
    keep style code out of the earlier codes. This  also gives me an opportunity to talk about the
    parallel trees that inhabit most rendering engines.

    A browser engine module often takes one tree as input, and produces a different but related tree
    as output. For example, Gecko's [layout code](https://wiki.mozilla.org/Gecko:Key_Gecko_Structures_And_Invariants)
    takes a DOM tree and produces a "frame tree", which is then used to build a "view tree".
    Blink and WebKit transform the DOM tree into a [render tree](https://www.chromium.org/developers/design-documents/gpu-accelerated-compositing-in-chrome/).
    Later stages in all these engines produce still more trees, including layer trees and widget trees.

    The pipeline for out tou browser engine will look something like this, after we complete  a few
    mode stages:
        HTML -> HTML Parser -> DOM   ->
                                        -> Style -> Style tree -> Layout -> Layout tree -> Painting -> Pixels
        CSS  -> CSS Parser  -> Rules ->

    In my implementation, each node in the DOM tree has exactly one node in the style tree.
    But in a more complicated pipeline stage, several input nodes could collapse into a single output node.
    Or an input node might expand into several output nodes, or be skipped completely.
    For example, the style tree could exclude elements whose [display](https://developer.mozilla.org/en-US/docs/Web/CSS/display)
    property is set to 'none'.
 */
/// A node with associated style data.
struct StyledNode<'a> {
    node: &'a Node, // pointer to a DOM node
    specified_values: PropertyMap,
    children: Vec<StyledNode<'a>>,
}

/// css `display`
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

    /// Return the specified value of property `name`,
    /// or property `fallback_name` if that doesn't exits,
    /// or value `default` if neither does.
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .unwrap_or_else(
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


/*
    Selector Matching

    The first step in building the style tree is [selector matching](https://www.w3.org/TR/CSS2/selector.html#pattern-matching).
    This will be very easy, since the [CSS parser](css.rs) supports only simple selectors.
    You can tell whether a simple selector matches an element just by looking at the element itself.
    Matching compound selectors would require traversing the DOM tree to look at the element's siblings, parents, etc.
 */

/// Selector matching
fn matches(element: &ElementData, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(s) => matches_simple_selector(element, s)
    }
}

/*
    To test whether a simple selector matches an element, just look at each selector component,
    and return false if the element doesn't have a matching class, ID, or tag name.

    Rust note: This function uses the [any](https://doc.rust-lang.org/core/iter/trait.Iterator.html#method.any)
    method, which returns true if an iterator contains an element that passes the provided test.
    This is the same as the [any](https://docs.python.org/3/library/functions.html#any)
    function in Python (or [Haskell](https://hackage.haskell.org/package/base-4.7.0.1/docs/Prelude.html#v:any))
    or the [some](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some)
    method in JavaScript.
 */

fn matches_simple_selector(element: &ElementData, selector: &SimpleSelector) -> bool {
    // Check type selector
    if selector.tag_name.iter().any(|name| element.tag_name != *name) {
        return false;
    }

    // Check ID selector
    if selector.id.iter().any(|id| element.id() != Some(id)) {
        return false;
    }

    // Check class selectors
    if selector.class.iter().any(|class| !element.classes().contains(class.as_str())) {
        return false;
    }

    // We didn't find any non-matching selector components.
    true
}
