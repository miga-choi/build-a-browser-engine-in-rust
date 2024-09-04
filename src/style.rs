use std::collections::HashMap;
use crate::css::Value;
use crate::dom::Node;


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