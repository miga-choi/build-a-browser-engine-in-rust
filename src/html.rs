use crate::dom;
use std::collections::HashMap;

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
 */


/*
    The parser stores its input string and a current position within the string.
    The position is the index of the next character we haven't processed yet.
 */

struct Parser {
    pos: usize, // "usize" is an unsigned integer, similar to "size_t" in C
    input: String,
}


/*
    We can use this to implement some simple methods for peeking at the next
    characters in the input.
 */

impl Parser {
    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Do the next characters start with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// If the exact string `s` is found at the current position, consume it. Otherwise, panic.
    fn expect(&mut self, s: &str) {
        if self.starts_with(s) {
            self.pos += s.len();
        } else {
            panic!("Expected {:?} at byte {} but it was not found", s, self.pos);
        }
    }


    /*
        Rust strings are stored as UTF-8 byte arrays. To go to the next character,
        we can't just advance by one byte.
     */

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let c = self.next_char();
        self.pos += c.len_utf8();
        c
    }


    /*
        Often we will want to consume a string of consecutive characters. The "consume_while"
        method consumes characters that meet a given condition, and returns them as a string.
        This method's argument is a function that takes a char and returns a bool.
     */

    /// Consume characters until `test` returns false.
    fn consume_while(&mut self, test: impl Fn(char) -> bool) -> String {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }


    /*
        We can use this to ignore a sequence of space characters, or to consume a string of
        alphanumeric characters.
     */

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Parse a tag or attribute name.
    fn parse_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }


    /*
        In our simplified version of HTML, a text node can contain any character except "<".
     */

    /// Parse a text node.
    fn parse_text(&mut self) -> dom::Node {
        dom::text(self.consume_while(|c| c != '<'))
    }


    /*
        Parsing attributes is pretty easy in our simplified syntax. Until we reach the
        end of the opening tag (>) we repeatedly look for a name followed by = and then
        a string enclosed in quotes.
     */

    /// Parse a quoted value
    fn parse_attribute_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '""' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        let close_quote = self.consume_char();
        assert_eq!(open_quote, close_quote);
        value
    }

    /// Parse a single name="value" pair.
    fn parse_attribute(&mut self) -> (String, String) {
        let name = self.parse_name();
        self.expect("=");
        let value = self.parse_attribute_value();
        (name, value)
    }

    /// Parse a list of name="value" pairs, seperated by whitespace.
    fn parse_attributes(&mut self) -> dom::AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attribute();
            attributes.insert(name, value);
        }
        attributes
    }

    /*
        An element is more complicated. It includes opening and closing tags, and between them
        any number of child nodes.
     */

    /// Parse a single element, including its open tag, contents, and closing tag.
    fn parse_element(&mut self) -> dom::Node {
        // Opening tag.
        self.expect("<");
        let tag_name = self.parse_name();
        let attrs = self.parse_attributes();
        self.expect(">");

        // Contents.
        let children = self.parse_nodes();

        // Closing tag.
        self.expect("</");
        self.expect(&tag_name);
        self.expect(">");

        dom::element(tag_name, attrs, children)
    }


    /*
        Now we're ready to start parsing HTML. To parse a single node, we look at its
        first character to see if it is an element or a text node.
     */

    /// Parse a single node.
    fn parse_node(&mut self) -> dom::Node {
        if self.starts_with("<") {
            self.parse_element()
        } else {
            self.parse_text()
        }
    }


    /*
        To parse the child nodes, we recursively call "parse_node" in a loop until
        we reach the closing tag. This function returns a Vec, which is Rust's name
        for a growable array.
     */

    /// Parse a sequence of sibling nodes.
    fn parse_nodes(&mut self) -> Vec<dom::Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}


/*
    This function will create a root node for the document if it doesn't
    include one explicitly; this is similar to what a real HTML parser does.
 */

/// Parse an HTML document and return the root element.
pub fn parse(source: String) -> dom::Node {
    let mut nodes = Parser { pos: 0, input: source }.parse_nodes();

    // If the document contains a root element, just return it. Otherwise, create one.
    if nodes.len() == 1 {
        nodes.remove(0)
    } else {
        dom::element("html".to_string(), HashMap::new(), nodes)
    }
}