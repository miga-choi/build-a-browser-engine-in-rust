//! Basic CSS block layout.

/*
    The layout module takes the style tree and translates it into a bunch of rectangles in
    a two-dimensional space.

    The layout module's input is the style tree, and its output is yet another tree, the
    "layout tree".
 */

/*
    Layout is all about "boxes". A box is a rectangular section of a web page. It has a
    width, a height, and a position on the page. This rectangle is called the "content area"
    because it's where the box's content is drawn. The content may be text, image, video,
    or other boxes.

    A box may also have padding, borders, and margins surrounding its content area. The CSS
    spec has a [diagram](https://www.w3.org/TR/CSS2/box.html#box-dimensions) showing how
    all these layers fit together.

    The engine stores a box's content area and surrounding areas in the following structure.

    Rust note: `f32` is a 32-bit floating point type.
 */

// CSS box model. All sizes are in px.

/// Position of the content area relative to the document origin:
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Surrounding edges:
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}
