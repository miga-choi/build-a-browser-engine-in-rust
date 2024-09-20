//! Basic CSS block layout.

use crate::{css, style};

/**
 *  the layout module takes the style tree and translates it into a bunch of rectangles in
 *  a two-dimensional space.
 *
 *  the layout module's input is the style tree, and its output is yet another tree, the
 *  "layout tree".
 */

/**
 *  layout is all about "boxes". a box is a rectangular section of a web page. It has a
 *  width, a height, and a position on the page. this rectangle is called the "content area"
 *  because it's where the box's content is drawn. The content may be text, image, video,
 *  or other boxes.
 *
 *  a box may also have padding, borders, and margins surrounding its content area. The CSS
 *  spec has a [diagram](https://www.w3.org/tr/css2/box.html#box-dimensions) showing how
 *  all these layers fit together.
 *
 *  the engine stores a box's content area and surrounding areas in the following structure.
 *
 *  rust note: `f32` is a 32-bit floating point type.
 */

// css box model. all sizes are in px.

/// position of the content area relative to the document origin:
#[derive(Copy)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// surrounding edges:
#[derive(Copy)]
struct EdgeSizes {
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
}

#[derive(Copy)]
struct Dimensions {
    content: Rect,
    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}


/*
    block and inline layout

    the css display property determines which type of box an element generates. CSS
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


/**
 *  To build the layout tree, we need to look at the display property for each DOM node.
 *  I added some code to the style module to get the display value for a node. If there's
 *  no specified value it returns the initial value, "inline".
 *
 *  see style::Display
 *  see style::StyledNode
 */


/**
 *  Now we can walk through the style tree, build a LayoutBox for each node, and then
 *  insert boxes for the node's children. If a node's display property is set to 'none'
 *  then it is not included in the layout tree.
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
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
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


impl LayoutBox {
    /**
     *  Traversing the Layout Tree
     *
     *  The entry point to this code is the layout function, which takes a LayoutBox
     *  and calculates its dimensions. We’ll break this function into three cases,
     *  and implement only one of them for now:
     */
    /// Lay out a box and its descendants.
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout(containing_block),
            BoxType::InlineNode(_) => {} // TODO
            BoxType::AnonymousBlock => {} // TODO
        }
    }

    /**
     *  A block's layout depends on the dimensions of its "containing block". For block boxes
     *  in normal flow, this is just the box's parent. For the root element, it's the size of
     *  the browser window (or "viewport").
     *
     *  you may remember from the previous article that a block's width depends on its parent,
     *  while its height depends on its children. This means that our code needs to traverse
     *  the tree top-down while calculating widths, so it can lay out the children after their
     *  parent's width is known, and traverse bottom-up to calculate heights, so that a parent's
     *  height is calculated after its children's.
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


    /**
     *  Calculating the Width
     *
     *  The width calculation is the first step in the block layout function, and also the
     *  most complicated. I'll walk through it step by step. To start, we need the values of
     *  the CSS width property and all the left and right edge sizes.
     */
    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style: style::StyledNode = self.get_style_node();

        // `width` has initial value `auto`.
        let auto: css::Value = css::Value::Keyword("auto".to_string());
        let mut width: css::Value = style.value("width").unwrap_or(auto.clone());

        // margin, border, and padding have initial value 0.
        let zero: css::Value = css::Value::Length(0.0, css::Unit::Px);


        /**
         *  This uses a helper function called "style::StyledNode::lookup", which just
         *  tries a series of values in sequence. If the first property isn't set, it
         *  tries the second one. If that's not set either, it returns the given default
         *  value. This provides an incomplete (but simple) implementation of [shorthand properties](https://www.w3.org/TR/CSS2/about.html#shorthand)
         *  and initial values.
         *
         *  Note: This is similar to the following code in, say, JavaScript or Ruby:
         *    margin_left = style["margin-left"] || style["margin"] || zero;
         */

        let mut margin_left: css::Value = style.lookup("margin-left", "margin", &zero);
        let mut margin_right: css::Value = style.lookup("margin-right", "margin", &zero);

        let border_left: css::Value = style.lookup("border-left-width", "border-width", &zero);
        let border_right: css::Value = style.lookup("border-right-width", "border-width", &zero);

        let padding_left: css::Value = style.lookup("padding-left", "padding", &zero);
        let padding_right: css::Value = style.lookup("padding-right", "padding", &zero);


        /**
         *  Since a child can't change its parent's width, it needs to make sure its own
         *  width fits the parent's. The CSS spec expresses this as a set of [constraints](https://www.w3.org/TR/CSS2/visudet.html#blockwidth)
         *  and an algorithm for solving them. The following code implements that algorithm.
         */

        /*
           First we add up the margin, padding, border, and content widths.
           The "css::Value:to_px" helper method converts lengths to their numerical values.
           If a property is set to "auto", it returns 0 so it doesn't affect the sum.

         */
        let total: f32 = [
            &margin_left, &margin_right,
            &border_left, &border_right,
            &padding_left, &padding_right,
            &width
        ].iter().map(|v: &&css::Value| v.to_px()).sum();

        /**
         *  This is the minimum horizontal space needed for the box. If this isn't equal
         *  to the container width, we'll need to adjust something to make it equal.
         */

        /*
            If the  width or margins are set to "auto", they can expand or contract to fit
            the available space. Following the spec, we first check if the box is too big.
            If so, we set any expandable margins to zero.
         */
        /// If width is not auto and the total is wider than the container,
        /// treat auto margins as 0.
        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = css::Value::Length(0.0, css::Unit::Px)
            }
            if margin_right == auto {
                margin_right = css::Value::Length(0.0, css::Unit::Px)
            }
        }

        /*
            If the box is too large for its container, it "overflows" the container.
            If it's too small, it will "underflow", leaving extra space. We'll calculate
            the underflow-the amount of extra space left in the container. (If this
            number is negative, it is actually an overflow.)
         */
        let underflow: f32 = containing_block.content.width - total;

        /*
            We now follow the spec's [algorithm](https://www.w3.org/TR/CSS2/visudet.html#blockwidth)
            for eliminating any overflow or underflow by adjusting the expandable dimensions.
            If there are no "auto" dimensions, we adjust the right margin. (Yes, this means
            the margin may be [negative](https://www.smashingmagazine.com/2009/07/the-definitive-guide-to-using-negative-margins/)
            in the case of an overflow!)
         */
        match (width == auto, margin_left == auto, margin_right == auto) {
            // If the values are overconstrained, calculate margin_right.
            (false, false, false) => {
                margin_right = css::Value::Length(margin_right.to_px() + underflow, css::Unit::Px);
            }

            // If exactly one size is auto, its used value follows from the equality.
            (false, false, true) => {
                margin_right = css::Value::Length(underflow, css::Unit::Px);
            }
            (false, true, false) => {
                margin_left = css::Value::Length(underflow, css::Unit::Px);
            }

            // If width is set to auto, any other auto values become 0.
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = css::Value::Length(0.0, css::Unit::Px);
                }
                if margin_right == auto {
                    margin_right = css::Value::Length(0.0, css::Unit::Px);
                }

                if underflow >= 0.0 {
                    // Expand width to fill the underflow.
                    width = css::Value::Length(underflow, css::Unit::Px);
                } else {
                    // Width can't be negative. Adjust the right margin instead.
                    width = css::Value::Length(0.0, css::Unit::Px);
                    margin_right = css::Value::Length(margin_right.to_px() + underflow, css::Unit::Px);
                }
            }

            // If margin-left and margin-right are both auto, their used values are equal.
            (false, true, true) => {
                margin_left = css::Value::Length(underflow / 2.0, css::Unit::Px);
                margin_right = css::Value::Length(underflow / 2.0, css::Unit::Px);
            }
        }

        /**
         *  At this point, the constraints are met and any "auto" values have been
         *  converted to lengths. The results are the [used values](https://www.w3.org/TR/CSS2/cascade.html#used-value)
         *  for the horizontal box dimensions, which we will store in the layout tree.
         */
    }

    /**
     *  Positioning
     *
     *  This function looks up the remaining margin/padding/border styles, and uses these
     *  along with the containing block dimensions to determine this block's position on
     *  the page.
     */
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style: style::StyledNode = self.get_style_node();
        let d: &mut Dimensions = &mut self.dimensions;

        // margin, border, and padding have initial value 0.
        let zero: css::Value = css::Value::Length(0.0, css::Unit::Px);

        // If margin-top or margin-bottom is "auto", the used value is zero.
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x +
            d.margin.left + d.border.left + d.padding.left;

        // Position the box below all the previous boxes in the container.
        d.content.y = containing_block.content.height + containing_block.content.y +
            d.margin.top + d.border.top + d.padding.top;
    }

    /**
     *  Children
     *
     *  Here's the code that recursively lays out the box's contents. As it loops through
     *  the child boxes, it keeps track of the total content height. This is used by the
     *  positioning code (above) to find the vertical position of the next child.
     */
    fn layout_block_children(&mut self) {
        for child in &mut self.children {
            child.layout(self.dimensions);
            // Increment the height so each child is laid out below the previous one.
            self.dimensions.content.height += child.dimensions.margin_box().height;
        }
    }
}
