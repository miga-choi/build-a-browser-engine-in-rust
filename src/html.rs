/*
    HTML has its own unique parsing algorithm. Unlike parsers from most programming
    languages and file formats, the HTML parsing algorithm does not reject invalid input.
    Instead, it includes specific error-handling instructions, so web browsers can agree
    on how to display every web page, even ones that don't conform to the syntax rules.
    Web browsers have to do this to be usable: Since non-conforming HTML has been supported
    since the early days of the web, it is now used in a huge portion of existing web page.

    A Simple HTML Dialect
        <html>
            <body>
                <h1>Title</h1>
                <div id="main" class="test">
                    <p>Hello <em>world</em>!</p>
                </div>
            </body>
        </html>

    The following syntax is allowed:
        1. Balanced tags: <p>...</p>
        2. Attributes with quoted values: id="main"
        3. Text nodes: <em>world</em>

    Everything else is unsupported, include
        1. Comments
        2. Doctype declarations
        3. Escaped characters (like &amp;) and CDATA sections
        4. Self-closing tags: <br /> or <br> with no closing tag
        5. Error handling (e.g. unbalanced or improperly nested tags)
        6. Namespaces and other XHTML syntax: <html:body>
        7. Character encoding detection

    The HTML parser structure is based loosely on the tokenizer module from
    Servo's CSS-Parser(https://github.com/servo/rust-cssparser) library.
    It has no real error handling; in most cases, it just aborts when faced
    with unexpected syntax.

    The parser stores its input string and a current position within the string.
    The position is the index of the next character we haven't processed yet.
 */
struct Parser {
    pos: usize, // "usize" is an unsigned integer, similar to "size_t" in C
    input: String,
}

/*
    We can use this to implement some simple methods fro peeking at the next
    characters in the input.
 */
impl Parser {
    // Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    // Do the next characters start with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    // If the exact string `s` is found at the current position, consume it.
    // Otherwise, panic.
    fn expect(&mut self, s: &str) {
        if self.starts_with(s) {
            self.pos += s.len();
        } else {
            panic!("Expected {:?} at byte {} but it was not found", s, self.pos);
        }
    }
}