//! Basic DOM data structures.

use std::collections::{HashMap, HashSet};


/*
    The DOM is a tree of nodes. A node has zero or more children. (It also has
    various other attributes and methods, but we can ignore most of those for now.)
 */

pub struct Node {
    // data specific to each node type
    pub node_type: NodeType,

    // data common to all nodes
    pub children: Vec<Node>,
}


/*
    There are several node types, but for now we will ignore most of them and say
    that a node is either an Element or a Text node. In a language with inheritance
    these would be subtypes of Node. In Rust, they can be enum (Rust's keyword for
    a "tagged union" or "sum type").
 */

/*
    e.g.
        NodeType {
            Text("..."),
            Element(ElementData),
        }
 */
pub enum NodeType {
    Element(Element),
    Text(String),
}


/*
    An element includes a tag name and any number of attributes, which can be stored
    as a map from names to values. This engine doesn't support namespaces, so it just
    stores tag and attribute names as simple strings.
 */

/*
    e.g.
        Element {
            tag_name: "div",
            attributes: AttributeMap,
        }
 */
pub struct Element {
    pub tag_name: String,
    pub attributes: AttributeMap,
}


/*
    e.g.
        "id": "...",
        "class": "...",
        "style": "..."
 */
pub type AttributeMap = HashMap<String, String>;


/*
    Some constructor functions to make it easy to create new codes:
 */

/// Return a new `Node` with `Text`
pub fn text(data: String) -> Node {
    Node {
        node_type: NodeType::Text(data),
        children: Vec::new(),
    }
}

/// Return a new `Node` with `Element`
pub fn element(tag_name: String, attributes: AttributeMap, children: Vec<Node>) -> Node {
    Node {
        node_type: NodeType::Element(Element { tag_name, attributes }),
        children,
    }
}


/*
    To help, we'll add some convenient ID and class accessors to out [DOM element type](dom.rs).
    The class attribute can contain multiple class names separated by spaces, which we return
    in a hash table.
 */

// Element methods

/// `Element` struct
impl Element {
    /// Return "ID" String or `None`
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    /// Return "Class" Set or empty Set
    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(class_list) => class_list.split(' ').collect(),
            Node => HashSet::new(),
        }
    }
}
