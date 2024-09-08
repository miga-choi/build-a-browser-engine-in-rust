//! Basic DOM data structures.

use std::collections::{HashMap, HashSet};


/*
    The DOM is a tree of nodes. A node has zero or more children. (It also has
    various other attributes and methods, but we can ignore most of those for now.)
 */

pub struct Node {
    // data common to all nodes
    pub children: Vec<Node>,

    // data specific to each node type
    pub node_type: NodeType,
}


/*
    There are several node types, but for now we will ignore most of them and say
    that a node is either an Element or a Text node. In a language with inheritance
    these would be subtypes of Node. In Rust, they can be enum (Rust's keyword for
    a "tagged union" or "sum type").
 */

// e.g. NodeType { Text("..."), Element(ElementData) }
pub enum NodeType {
    Text(String),
    Element(ElementData),
}


/*
    An element includes a tag name and any number of attributes, which can be stored
    as a map from names to values. This engine doesn't support namespaces, so it just
    stores tag and attribute names as simple strings.
 */

/*
    Default ElementData

    e.g.
        ElementData { "tag_name": "p", attrs: AttrMap }
 */
pub struct ElementData {
    pub tag_name: String,
    pub attributes: AttributeMap,
}

// e.g. { "class": "...", "style": "..."}
pub type AttributeMap = HashMap<String, String>;


/*
    Some constructor functions to make it easy to create new codes:
 */

pub fn text(data: String) -> Node {
    Node {
        children: Vec::new(),
        node_type: NodeType::Text(data),
    }
}

pub fn element(tag_name: String, attributes: AttributeMap, children: Vec<Node>) -> Node {
    Node {
        children,
        node_type: NodeType::Element(ElementData { tag_name, attributes }),
    }
}


/*
    To help, we'll add some convenient ID and class accessors to out [DOM element type](dom.rs).
    The class attribute can contain multiple class names separated by spaces, which we return
    in a hash table.
 */

// Element methods

// Implemented ElementData based on Default ElementData
impl ElementData {
    pub fn id(&self) -> Option<&String> {
        self.attributes.get("id")
    }

    pub fn classes(&self) -> HashSet<&str> {
        match self.attributes.get("class") {
            Some(classlist) => classlist.split(' ').collect(),
            Node => HashSet::new()
        }
    }
}
