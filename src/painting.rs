use crate::{css, layout};

/**
 *  Painting 101
 *
 *  This code takes the tree of boxes from the layout module and turns them into an array
 *  of pixels. This process is also known as "rasterization".
 *
 *  HTML -> HTML parser
 *                      --- DOM   --->
 *                                      Style --- Style tree ---> Layout --- Layout tree ---> Painting --- Pixels --->
 *                      --- Rules --->
 *  CSS  -> CSS parser
 *
 *  Browser usually implement rasterization with the help of graphics APIs and libraries
 *  like Skia, Cairo, Direct2D, and so on. These APIs provide functions for painting
 *  polygons, lines, curves, gradients, and text. For now, I'm going to write my own
 *  rasterizer that can only paint one thing: rectangles.
 *
 *  Eventually I want to implement text rendering. At that point, I may throw away this
 *  toy painting code and switch to a "real" 2D graphics library. But for now, rectangles
 *  are sufficient to turn the output of my block layout algorithm into pictures.
 */

/**
 *  Building the Display List
 *
 *  Before painting, we will walk through the layout tree and build a [display list](https://en.wikipedia.org/wiki/Display_list).
 *  This is a list of graphics operations like "draw a circle" or "draw a string of text".
 *  Or in our case, just "draw a rectangle".
 *
 *  Why put commands into a display list, rather than execute them immediately? The display
 *  list is useful for a several reasons. You can search it for items that will be completely
 *  covered up by later operations, and remove them to eliminate wasted painting. You can
 *  modify and re-use the same display list to generate different types of output: for example,
 *  pixels for displaying on a screen, or vector graphics for sending to a printer.
 *
 *  The Engine's display list is a vector of DisplayCommands. For now there is only one
 *  type of DisplayCommand, a solid-color rectangle:
 */
type DisplayList = Vec<DisplayCommand>;

enum DisplayCommand {
    SolidColor(css::Color, layout::Rect)
}


/**
 *  To build the display list, we walk through the layout tree and generate a series of
 *  commands for each box. First we draw the box's background, then we draw its borders
 *  and content on top of the background.
 */

fn build_display_list(layout_root: &layout::LayoutBox) -> DisplayList {
    let mut list: Vec<DisplayCommand> = Vec::new();
    render_layout_box(&mut list, layout_root);
    list
}

fn render_layout_box(list: &mut DisplayList, layout_box: &layout::LayoutBox) {
    render_background(list, layout_box);
    render_borders(list, layout_box);

    // TODO: render text

    for child in &layout_box.children {
        render_layout_box(list, child);
    }
}


/**
 *  By default, HTML elements are stacked in the order they appear: If two elements overlap,
 *  the later one is drawn on top of the earlier one. This is reflected in our display list,
 *  which will draw the elements in the same order they appear in the DOM tree. If this code
 *  supported the [z-index](https://www.w3.org/TR/CSS2/visuren.html#z-index) property, then
 *  individual elements would be able to override this stacking order, and we'd need to sort
 *  the display list accordingly.
 *
 *  The background is easy. It's just solid rectangle. If no background color is specified,
 *  then the background is transparent and we don't need to generate a display command.
 */

fn render_background(list: &mut DisplayList, layout_box: &layout::LayoutBox) {
    get_color(layout_box, "background")
        .map(
            |color: css::Color| list.push(
                DisplayCommand::SolidColor(color, layout_box.dimensions.border_box())
            )
        );
}

/// Return the specified color for CSS property `name`, or None if no color was specified.
fn get_color(layout_box: &layout::LayoutBox, name: &str) -> Option<css::Color> {
    match layout_box.box_type {
        layout::BoxType::BlockNode(style) | layout::BoxType::InlineNode(style) => match style.value(name) {
            Some(css::Value::ColorValue(color)) => Some(color),
            _ => None,
        },
        layout::BoxType::AnonymousBlock => None,
    }
}


/**
 *  The borders are similar, but instead of a single rectangle we draw four-one for
 *  each edge of the box.
 */
fn render_borders(list: &mut DisplayList, layout_box: &layout::LayoutBox) {
    let color: css::Color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return, // bail out if no border-color is specified
    };

    let d: &layout::Dimensions = &layout_box.dimensions;
    let border_box: layout::Rect = d.border_box();

    // Top border
    list.push(DisplayCommand::SolidColor(color, layout::Rect {
        x: border_box.x,
        y: border_box.y,
        width: border_box.width,
        height: d.border.top,
    }));

    // Right border
    list.push(DisplayCommand::SolidColor(color, layout::Rect {
        x: border_box.x + border_box.width - d.border.right,
        y: border_box.y,
        width: d.border.left,
        height: border_box.height,
    }));

    // Bottom border
    list.push(DisplayCommand::SolidColor(color, layout::Rect {
        x: border_box.x,
        y: border_box.y + border_box.height - d.border.bottom,
        width: border_box.width,
        height: d.border.bottom,
    }));

    // Left border
    list.push(DisplayCommand::SolidColor(color, layout::Rect {
        x: border_box.x,
        y: border_box.y,
        width: d.border.left,
        height: border_box.height,
    }));
}


/**
 *  Rasterization
 *
 *  Now that we've built the display list, we need to turn it into pixels by executing
 *  each DisplayCommand. We'll store the pixels in a Canvas:
 */
struct Canvas {
    pixels: Vec<css::Color>,
    width: usize,
    height: usize,
}

impl Canvas {
    // Create a blank canvas
    fn new(width: usize, height: usize) -> Canvas {
        let white = css::Color { r: 255, g: 255, b: 255, a: 255 };
        Canvas {
            pixels: vec![white; width * height],
            width,
            height,
        }
    }
}