//! Basic CSS block layout.

use crate::{css, style};
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


/*
    Block and Inline Layout

    The CSS display property determines which type of box an element generates. CSS
    defines several box types, each with its own layout rules. Here only going to talk
    about two of them: "block" and "inline".

    I'll use this bit of pseudo-HTML to illustrate the difference:
        <container>
            <a></a>
            <b></b>
            <c></c>
            <d></d>
        </container>
 */


/*
    A box can be a block node, an inline node, or an anonymous block box. (This will
    need to change when I implement text layout, because line wrapping can cause a
    single inline node to split into multiple boxes. But it will do for now.)
 */
enum BoxType<'a> {
    BlockNode(&'a style::StyledNode<'a>),
    InlineNode(&'a style::StyledNode<'a>),
    AnonymousBlock,
}


/// A node in the layout tree.
/*
    The Layout Tree

    The layout tree is a collection of boxes. A box has dimensions, and it may contain
    child boxes.
 */
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}


/*
    To build the layout tree, we need to look at the display property for each DOM node.
    I added some code to the style module to get the display value for a node. If there's
    no specified value it returns the initial value, "inline".

    see style::Display
    see style::StyledNode
 */


/*
    Now we can walk through the style tree, build a LayoutBox for each node, and then
    insert boxes for the node's children. If a node's display property is set to 'none'
    then it is not included in the layout tree.
 */

/// Build the tree of LayoutBoxes, but don't perform any layout calculations yet.
fn build_layout_tree<'a>(style_node: &'a style::StyledNode<'a>) -> LayoutBox<'a> {
    // Create the root box.
    let mut root: LayoutBox = LayoutBox::new(match style_node.display() {
        style::Display::Block => BoxType::BlockNode(style_node),
        style::Display::Inline => BoxType::InlineNode(style_node),
        style::Display::None => panic!("Root node has display: none.")
    });

    // Create the descendant boxes.
    for child in &style_node.children {
        match child.display() {
            style::Display::Block => root.children.push(build_layout_tree(child)),
            style::Display::Inline => root.get_inline_container().children.push(build_layout_tree(child)),
            style::Display::None => {} // Don't lay out nodes with `display: none;`
        }
    }

    root
}

impl LayoutBox {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type,
            dimensions: Default::default(), // initially set all fields to 0.0
            children: Vec::new(),
        }
    }

    /*
        If a block node contains an inline child, create an anonymous block box to
        contain it. If there are several inline children in a row, put them all in
        the same anonymous container.
     */
    /// Where a new inline child should go.
    fn get_inline_container(&mut self) -> &mut LayoutBox {
        match self.box_type {
            BoxType::InlineNode(_) | AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                // If we've just generated an anonymous block box, keep using it.
                // Otherwise, create a new one.
                match self.children.last() {
                    Some(&LayoutBox { box_type: AnonymousBlock, .. }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }
}


/*
    Traversing the Layout Tree

    The entry point to this code is the layout function, which takes a LayoutBox and
    calculates its dimensions. We'll break this function into three cases, and implement
    only one of them for now:
 */
impl LayoutBox {
    /*
        Traversing the Layout Tree

        The entry point to this code is the layout function, which takes a LayoutBox
        and calculates its dimensions. We’ll break this function into three cases,
        and implement only one of them for now:
     */
    /// Lay out a box and its descendants.
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout(containing_block),
            BoxType::InlineNode(_) => {} // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    /*
        A block's layout depends on the dimensions of its "containing block". For block boxes
        in normal flow, this is just the box's parent. For the root element, it's the size of
        the browser window (or "viewport").

        You may remember from the previous article that a block's width depends on its parent,
        while its height depends on its children. This means that our code needs to traverse
        the tree top-down while calculating widths, so it can lay out the children after their
        parent's width is known, and traverse bottom-up to calculate heights, so that a parent's
        height is calculated after its children's.
     */
    fn layout_block(&mut self, containing_block: Dimensions) {
        // Child width can depend on parent width, so we need to
        // calculate this box's width before laying out its children.
        self.calculate_block_width(containing_block);

        // Determine where the box is located within its container.
        self.calculate_block_position(containing_block);

        // Recursively lay out the children of this box.
        self.layout_block_children();

        // Parent height can depend on child height, so `calculate_height`
        // must be called *after* the children are laid out.
        self.calculate_block_height();
    }


    /*
        Calculating the Width

        The width calculation is the first step in the block layout function, and also the
        most complicated. I'll walk through it step by step. To start, we need the values of
        the CSS width property and all the left and right edge sizes.
     */
    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style: style::StyledNode = self.get_style_node();

        // `width` has initial value `auto`.
        let auto: css::Value = css::Value::Keyword("auto".to_string());
        let mut width: css::Value = style.value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0.
        let zero: css::Value = css::Value::Length(0.0, css::Unit::Px);


        /*
            This uses a helper function called style::StyledNode::lookup, which just tries a
            series of values in sequence. If the first property isn't set, it tries the second
            one. If that's not set either, it returns the given default value. This provides an
            incomplete (but simple) implementation of [shorthand properties](https://www.w3.org/TR/CSS2/about.html#shorthand)
            and initial values.

            Note: This is similar to the following code in, say, JavaScript or Ruby:
                margin_left = style["margin-left"] || style["margin"] || zero;
         */

        let mut margin_left: css::Value = style.lookup("margin-left", "margin", &zero);
        let mut margin_right: css::Value = style.lookup("margin-right", "margin", &zero);

        let border_left: css::Value = style.lookup("border-left-width", "border-width", &zero);
        let border_right: css::Value = style.lookup("border-right-width", "border-width", &zero);

        let padding_left: css::Value = style.lookup("padding-left", "padding", &zero);
        let padding_right: css::Value = style.lookup("padding-right", "padding", &zero);
    }
}
