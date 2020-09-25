mod parse_error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum ParseError {
        MissingElement {
            selector: &'static str,
        },
        MissingAttribute {
            attribute: &'static str,
        },
        #[snafu(context(false))]
        MalformedUrl {
            source: url::ParseError,
        },
        IncorrectUrl,
        #[snafu(context(false))]
        InvalidInteger {
            source: std::num::ParseIntError,
        },
        #[snafu(context(false))]
        InvalidDate {
            source: chrono::ParseError,
        },
        UnknownRating {
            text: String,
        },
        InvalidDepth {
            style: String,
        },
        #[snafu(display("adult/mature content is currently blocked"))]
        Nsfw,
    }
}

pub mod header;
pub mod view;

use chrono::NaiveDateTime;

use scraper::{ElementRef, Html, Selector};

pub use self::parse_error::ParseError;

use snafu::OptionExt;

use url::Url;

// TODO: Implement std::error::Error
#[derive(Debug)]
pub struct UnauthenticatedError;
impl std::error::Error for UnauthenticatedError {}

impl std::fmt::Display for UnauthenticatedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Unauthenticated")
    }
}

pub trait FromHtml: Sized {
    fn from_html(url: Url, document: &Html) -> Result<Self, ParseError>;
}

fn datetime(elem: ElementRef) -> Result<NaiveDateTime, ParseError> {
    let txt = match attr(elem, "title") {
        Ok(title) => title.to_string(),
        _ => text(elem),
    };

    Ok(NaiveDateTime::parse_from_str(&txt, "%b %e, %Y %I:%M %p")?)
}

fn number(elem: ElementRef) -> Result<u64, ParseError> {
    Ok(text(elem).parse()?)
}

fn text(elem: ElementRef) -> String {
    elem.text().map(str::trim).collect::<Vec<_>>().join(" ")
}

fn attr<'a>(
    elem: ElementRef<'a>,
    attribute: &'static str,
) -> Result<&'a str, ParseError> {
    elem.value()
        .attr(attribute)
        .context(parse_error::MissingAttribute { attribute })
}

fn select_first_elem<'a>(
    elem: ElementRef<'a>,
    css: &'static str,
) -> Result<ElementRef<'a>, ParseError> {
    // TODO; select_first and select_first_elem can probably be combined.
    let sel = Selector::parse(css).expect("invalid selector");
    elem.select(&sel)
        .next()
        .context(parse_error::MissingElement { selector: css })
}

fn select_first<'a>(
    document: &'a Html,
    css: &'static str,
) -> Result<ElementRef<'a>, ParseError> {
    let sel = Selector::parse(css).expect("invalid selector");
    document
        .select(&sel)
        .next()
        .context(parse_error::MissingElement { selector: css })
}

// TODO: Create a AsUserRef or somesuch trait that can be used to fetch a user

#[derive(Debug, Clone)]
pub struct MiniUser {
    avatar: Url,
    name: String,
    slug: String,
}

impl MiniUser {
    pub fn avatar(&self) -> &Url {
        &self.avatar
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
