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