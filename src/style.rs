//! Code for applying CSS styles to the DOM.
//!
//! I will call it "CSS Renderer"

use crate::css;
use crate::dom;
use std::collections::HashMap;


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
pub type PropertyMap = HashMap<String, css::Value>;


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
    pub node: &'a dom::Node,
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
    pub fn value(&self, name: &str) -> Option<css::Value> {
        self.specified_values.get(name).cloned()
    }

    /// Return the specified value of property `name`, or property `fallback_name`
    /// if that doesn't exist, or value `default` if neither does.
    pub fn lookup(&self, name: &str, fallback_name: &str, default: &css::Value) -> css::Value {
        self.value(name).unwrap_or_else(
            || self.value(fallback_name).unwrap_or_else(|| default.clone())
        )
    }

    /// The value of the `display` property (defaults to inline).
    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(css::Value::Keyword(s)) => match &*s {
                "block" => Display::Block,
                "none" => Display::None,
                _ => Display::Inline,
            },
            _ => Display::Inline,
        }
    }
}


/*
    The first step in building the style tree is [selector matching](https://www.w3.org/TR/CSS2/selector.html#pattern-matching).
    This will be very easy, since my CSS parser supports only simple selectors.
    You can tell whether a simple selector matches an element just by looking at
    the element itself. Matching compound selectors would require traversing
    the DOM tree to look at the element’s siblings, parents, etc.
 */
/// Selector matching:
fn matches(element: &dom::Element, selector: &css::Selector) -> bool {
    match selector {
        css::Selector::Simple(s) => matches_simple_selector(element, s)
    }
}


/*
    To test whether a simple selector matches an element, just look at each selector
    component, and return false if the element doesn't have a matching class, id, or
    tag name.

    Rust node: This function uses the [any](https://doc.rust-lang.org/core/iter/trait.Iterator.html#method.any)
    method, which returns true if an iterator contains an element that passes the
    provided test. This is the same as the [any](https://docs.python.org/3/library/functions.html#any)
    function in Python (or [Haskel](https://hackage.haskell.org/package/base-4.7.0.1/docs/Prelude.html#v:any)),
    or the [some](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/some)
    method in JavaScript.
 */
fn matches_simple_selector(element: &dom::Element, selector: &css::SimpleSelector) -> bool {
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


/*
    Building the Style Tree

    Next we need to traverse the DOM tree. For each element in the tree, we will search
    the stylesheet for matching rules.

    When comparing two rules that match the same element, we need to use the highest-specificity
    selector from each match. Because our CSS parser stores the selectors from most- to
    least-specific, we can stop as soon as we find a matching one, and return its
    specificity along with a pointer to the rule.
 */

/// A single CSS rule and the specificity of its most specific matching selector.
type MatchedRule<'a> = (css::Specificity, &'a css::Rule);

/// If `rule` matches `element`, return a `MatchedRule`. Otherwise return `None`.
fn match_rule<'a>(element: &dom::Element, rule: &'a css::Rule) -> Option<MatchedRule<'a>> {
    // Find the first (most specific) matching selector.
    rule.selectors
        .iter().find(|selector: &&css::Selector| matches(element, selector))
        .map(|selector: &css::Selector| (selector.specificity(), rule))
}


/*
    To find all the rules that match an element we call `filter_map`, which does a linear
    scan through the style sheet, checking every rule and throwing out ones that don't match.
    A real browser engine would speed this up by storing the rules in multiple hash tables
    based on tag name, id, class, etc.
 */
/// Find all CSS rules that match the given element.
fn matching_rules<'a>(element: &dom::Element, stylesheet: &'a css::Stylesheet) -> Vec<MatchedRule<'a>> {
    // For now, we just do a linear scan of all the rules. For large documents,
    // it would be more efficient to store the rules in hash tables based on
    // tag name, id, class, etc.
    stylesheet.rules.iter().filter_map(|rule: &css::Rule| match_rule(element, rule)).collect()
}


/*
    Once we have the matching rules, we can find the "specified values" for the element.
    We insert each rule's property values into a HashMap. We sort the matches by specificity,
    so the more-specific rules are processed after the less-specific ones, and can overwrite
    their values in the HashMap.
 */
/// Apply styles to a single element, returning the specified values.
fn specified_values(element: &dom::Element, stylesheet: &css::Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(element, stylesheet);

    // Go through the rules from lowest to highest specificity.
    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    values
}


/*
    Now we have everything we need to walk through the DOM tree and build the style tree.
    Note that selector matching works only on elements, so the specified values for
    a text node are just and empty map.
 */
/// Apply a stylesheet to an entire DOM tree, returning a `StyledNode` tree.
pub fn style_tree<'a>(root: &'a dom::Node, stylesheet: &'a css::Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            dom::NodeType::Element(ref element) => specified_values(element, stylesheet),
            dom::NodeType::Text(_) => HashMap::new(),
        },
        children: root.children.iter().map(|child: &dom::Node| style_tree(child, stylesheet)).collect(),
    }
}
