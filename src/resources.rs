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
        #[snafu(context(false))]
        Json {
            source: serde_json::Error,
        },
        #[snafu(display("adult/mature content is currently blocked"))]
        Nsfw,
    }
}

pub mod header;
pub mod msg;
pub mod view;

use chrono::NaiveDateTime;

use scraper::{ElementRef, Html, Selector};

pub use self::parse_error::ParseError;

use snafu::OptionExt;

use std::fmt;
use std::str::FromStr;

use url::Url;

#[derive(Debug)]
pub struct UnauthenticatedError;
impl std::error::Error for UnauthenticatedError {}

impl fmt::Display for UnauthenticatedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
pub enum Rating {
    General,
    Mature,
    Adult,
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let txt = match self {
            Rating::General => "General",
            Rating::Mature => "Mature",
            Rating::Adult => "Adult",
        };

        write!(f, "{}", txt)
    }
}

impl FromStr for Rating {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, ParseError> {
        match text {
            "Adult" => Ok(Rating::Adult),
            "Mature" => Ok(Rating::Mature),
            "General" => Ok(Rating::General),
            _ => Err(ParseError::UnknownRating { text: text.into() }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Submission {
    view_id: u64,
    preview: Url,
    rating: Rating,
    title: String,
    description: String,
    artist: MiniUser,
}

impl From<Submission> for crate::keys::ViewKey {
    fn from(sub: Submission) -> Self {
        Self {
            view_id: sub.view_id,
        }
    }
}

impl From<&Submission> for crate::keys::ViewKey {
    fn from(sub: &Submission) -> Self {
        Self {
            view_id: sub.view_id,
        }
    }
}

impl Submission {
    pub fn preview(&self) -> &Url {
        &self.preview
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn rating(&self) -> Rating {
        self.rating
    }

    pub fn artist(&self) -> &MiniUser {
        &self.artist
    }
}

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
