use ego_tree::iter::Edge;
use ego_tree::NodeRef;

use htmlescape::encode_minimal;

use scraper::node::{Element, Text};
use scraper::{ElementRef, Node};

use selectors::attr::CaseSensitivity;

use url::Url;

pub fn simplify(root: &Url, elem: ElementRef) -> String {
    let mut output = String::new();

    for edge in elem.traverse().skip(1) {
        match edge {
            Edge::Open(node) => simplify_open(root, &mut output, node),
            Edge::Close(node) => simplify_close(&mut output, node),
        }
    }

    output
}

fn simplify_open(root: &Url, output: &mut String, node: NodeRef<Node>) {
    match node.value() {
        Node::Comment(_) => (),
        Node::Document => (),
        Node::Fragment => (),
        Node::Doctype(_) => (),
        Node::Text(txt) => simplify_open_text(output, txt),
        Node::Element(elem) => simplify_open_element(root, output, elem),
        Node::ProcessingInstruction(_) => eprintln!("OPEN pi"),
    }
}

fn simplify_open_element(root: &Url, output: &mut String, elem: &Element) {
    match elem.name() {
        "strong" | "b" | "em" | "i" | "u" | "s" | "code" | "hr" | "span"
        | "div" => bbcode_open(output, elem),

        "br" => output.push_str("<br>"),

        "a" => bbcode_open_a(root, output, elem),
        "img" => bbcode_img(root, output, elem),

        _ => (),
    }
}

fn bbcode_img(root: &Url, output: &mut String, elem: &Element) {
    if let Some(src) = elem.attr("src").and_then(|h| root.join(h).ok()) {
        // TODO: Get alt text
        // TODO: Don't break non-avatar images
        // TODO: Qt can't handle escaped entities in rich text...
        let attr = encode_minimal(&src.to_string());
        let tag = format!(
            r#"<img width="50" height="50" align="middle" src="{}">"#,
            attr
        );
        output.push_str(&tag);
    }
}

fn bbcode_open_a(root: &Url, output: &mut String, elem: &Element) {
    match elem.attr("href").and_then(|h| root.join(h).ok()) {
        Some(url) => {
            // TODO: Qt can't handle escaped entities in rich text...
            let attr = encode_minimal(&url.to_string());
            let tag = format!(r#"<a href="{}">"#, attr);
            output.push_str(&tag);
        }
        None => {
            output.push_str("<a>");
        }
    }
}

const BBCODE_CLASSES: &[(&str, &str, Option<&str>)] = &[
    ("bbcode_hr", "<hr>", None),
    ("bbcode_b", "<strong>", Some("</strong>")),
    ("bbcode_i", "<em>", Some("</em>")),
    ("bbcode_u", "<u>", Some("</u>")),
    ("bbcode_s", "<s>", Some("</s>")),
    ("bbcode_left", r#"<div align="left">"#, Some("</div>")),
    ("bbcode_center", r#"<div align="center">"#, Some("</div>")),
    ("bbcode_right", r#"<div align="right">"#, Some("</div>")),
    (
        "bbcode_quote",
        r#"<blockquote class="quote">"#,
        Some("</blockquote>"),
    ),
    (
        "bbcode_quote_name",
        r#"<strong class="quote-name">"#,
        Some("</strong>"),
    ),
];

fn bbcode_close(output: &mut String, elem: &Element) {
    for (class, _, close) in BBCODE_CLASSES {
        if elem.has_class(class, CaseSensitivity::AsciiCaseInsensitive) {
            if let Some(tag) = close {
                output.push_str(tag);
            }
            return;
        }
    }

    let name = elem.name();
    if (name.eq_ignore_ascii_case("div") || name.eq_ignore_ascii_case("span"))
        && bbcode_span_color(elem).is_some()
    {
        output.push_str("</font>");
    }
}

fn bbcode_open(output: &mut String, elem: &Element) {
    for (class, open, _) in BBCODE_CLASSES {
        if elem.has_class(class, CaseSensitivity::AsciiCaseInsensitive) {
            output.push_str(open);
            return;
        }
    }

    let name = elem.name();
    if name.eq_ignore_ascii_case("div") || name.eq_ignore_ascii_case("span") {
        if let Some(color) = bbcode_span_color(elem) {
            // TODO: Qt can't handle escaped entities in rich text...
            let tag = format!(r#"<font color="{}">"#, encode_minimal(color));
            output.push_str(&tag);
        }
    }
}

fn bbcode_span_color(elem: &Element) -> Option<&str> {
    let style = match elem.attr("style") {
        Some(s) => s,
        None => return None,
    };

    if !style.starts_with("color: ") {
        return None;
    }

    if !style.ends_with(';') {
        return None;
    }

    Some(&style[7..style.len() - 1])
}

fn simplify_open_text(output: &mut String, text: &Text) {
    output.push_str(&encode_minimal(&text.text));
}

fn simplify_close(output: &mut String, node: NodeRef<Node>) {
    let elem = match node.value() {
        Node::Element(e) => e,
        _ => return,
    };

    match elem.name() {
        "strong" | "b" | "em" | "i" | "u" | "s" | "code" | "hr" | "span"
        | "div" => bbcode_close(output, elem),

        "a" => output.push_str("</a>"),

        _ => (),
    }
}

#[cfg(test)]
mod tests {
    use scraper::{Html, Selector};

    use super::*;

    fn html() -> Html {
        let txt = r#"
        <!DOCTYPE html>
        <html>
            <head></head>
            <body>
                <div id="escape-text">hello&amp;world</div>
                <div id="split-text">hello<p>world</p></div>
                <div id="bold"><strong class="bbcode bbcode_b">bold</strong></div>
                <div id="italic"><i class="bbcode bbcode_i">italic</i></div>
                <div id="under"><u class="bbcode bbcode_u">under</u></div>
                <div id="strike"><s class="bbcode bbcode_s">strike</s></div>
                <div id="left"><code class="bbcode bbcode_left">left</code></div>
                <div id="right"><code class="bbcode bbcode_right">right</code></div>
                <div id="center"><code class="bbcode bbcode_center">center</code></div>
                <div id="quote"><span class="bbcode bbcode_quote"><span class="bbcode_quote_name">name</span>content</span></div>
                <div id="rule"><hr class="bbcode bbcode_hr"></div>
                <div id="anchor"><a href="/view/1/&quot;">anchor</a></div>
                <div id="color"><span class="bbcode" style="color: red;">red</span></div>
                <div id="color-hex"><span class="bbcode" style="color: #0000FF;">blue</span></div>
            </body>
        </html>
        "#;

        Html::parse_document(txt)
    }

    fn do_simplify(selector: &str) -> String {
        let html = html();
        let selector = Selector::parse(selector).unwrap();
        let elem = html.select(&selector).next().unwrap();
        let root = Url::parse("https://www.furaffinity.net/view/1/").unwrap();
        simplify(&root, elem).trim().to_string()
    }

    #[test]
    fn simplify_escape_text() {
        let actual = do_simplify("#escape-text");
        assert_eq!(actual, "hello&amp;world");
    }

    #[test]
    fn simplify_split_text() {
        let actual = do_simplify("#split-text");
        assert_eq!(actual, "helloworld");
    }

    #[test]
    fn simplify_bold() {
        let actual = do_simplify("#bold");
        assert_eq!(actual, "<strong>bold</strong>");
    }

    #[test]
    fn simplify_italic() {
        let actual = do_simplify("#italic");
        assert_eq!(actual, "<em>italic</em>");
    }

    #[test]
    fn simplify_underline() {
        let actual = do_simplify("#under");
        assert_eq!(actual, "<u>under</u>");
    }

    #[test]
    fn simplify_strike() {
        let actual = do_simplify("#strike");
        assert_eq!(actual, "<s>strike</s>");
    }

    #[test]
    fn simplify_left() {
        let actual = do_simplify("#left");
        assert_eq!(actual, r#"<div align="left">left</div>"#);
    }

    #[test]
    fn simplify_right() {
        let actual = do_simplify("#right");
        assert_eq!(actual, r#"<div align="right">right</div>"#);
    }

    #[test]
    fn simplify_center() {
        let actual = do_simplify("#center");
        assert_eq!(actual, r#"<div align="center">center</div>"#);
    }

    #[test]
    fn simplify_quote() {
        let actual = do_simplify("#quote");
        let exp = r#"<blockquote class="quote"><strong class="quote-name">name</strong>content</blockquote>"#;
        assert_eq!(actual, exp);
    }

    #[test]
    fn simplify_rule() {
        let actual = do_simplify("#rule");
        assert_eq!(actual, "<hr>");
    }

    #[test]
    fn simplify_anchor() {
        let actual = do_simplify("#anchor");
        let exp =
            r#"<a href="https://www.furaffinity.net/view/1/%22">anchor</a>"#;
        assert_eq!(actual, exp);
    }

    #[test]
    fn simplify_color() {
        let actual = do_simplify("#color");
        let exp = r#"<font color="red">red</font>"#;
        assert_eq!(actual, exp);
    }

    #[test]
    fn simplify_color_hex() {
        let actual = do_simplify("#color-hex");
        let exp = r##"<font color="#0000FF">blue</font>"##;
        assert_eq!(actual, exp);
    }
}
