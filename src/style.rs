//! Code for applying CSS styles to the DOM.

use crate::dom::Node;
use std::collections::HashMap;


/// Map from CSS property names to values.
/*
    e.g.
        PropertyMap { "display": "block" }
 */
pub type PropertyMap = HashMap<String, String>;


/// A node with associated style data.
/*
    e.g.
        StyledNode {
            node: Node,
            specified_values: PropertyMap,
            children: Vec<StyleNode<'a>>,
 */
pub struct StyleNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyleNode<'a>>,
}


/// CSS's `display` property
pub enum Display {
    Inline, // display: inline
    Block,  // display: block
    None,   // display: none
}
