use crate::keys::SubmissionsKey;

use scraper::{Html, Selector};

use serde::Deserialize;

use snafu::{ensure, OptionExt};

use super::super::{
    attr, parse_error, select_first, select_first_elem, text, FromHtml,
    MiniUser, ParseError, Rating, Submission,
};

use std::collections::HashMap;
use std::convert::TryFrom;

use url::Url;

#[derive(Debug, Deserialize)]
struct SubInfo {
    title: String,
    description: String,
    username: String,
    lower: String,
    avatar_mtime: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Order::Ascending => "old",
            Order::Descending => "new",
        }
    }
}

#[derive(Debug)]
pub struct Submissions {
    items: Vec<Submission>,
    next: Option<SubmissionsKey>,
    prev: Option<SubmissionsKey>,
}

impl Submissions {
    pub fn next(&self) -> Option<&SubmissionsKey> {
        self.next.as_ref()
    }

    pub fn prev(&self) -> Option<&SubmissionsKey> {
        self.prev.as_ref()
    }

    pub fn items(&self) -> &[Submission] {
        self.items.as_slice()
    }

    pub fn into_items(self) -> Vec<Submission> {
        self.items
    }

    fn extract_nav(
        url: &Url,
        doc: &Html,
        css: &'static str,
    ) -> Result<SubmissionsKey, ParseError> {
        let elem = select_first(doc, css)?;
        let href = attr(elem, "href")?;
        let url = url.join(href)?;

        let key = SubmissionsKey::try_from(url)
            .map_err(|_| ParseError::IncorrectUrl)?;

        Ok(key)
    }
}

impl FromHtml for Submissions {
    fn from_html(url: Url, doc: &Html) -> Result<Self, ParseError> {
        let prev_res = Self::extract_nav(
            &url,
            doc,
            "a.button.prev[href^='/msg/submissions/'][href*='~']",
        );

        let prev = match prev_res {
            Ok(p) => Some(p),
            Err(ParseError::MissingElement { .. }) => None,
            Err(e) => return Err(e),
        };

        let next_res = Self::extract_nav(
            &url,
            doc,
            "a.button:not(.prev)[href^='/msg/submissions/'][href*='~']",
        );
        let next = match next_res {
            Ok(n) => Some(n),
            Err(ParseError::MissingElement { .. }) => None,
            Err(e) => return Err(e),
        };

        let script_sel = Selector::parse("script").unwrap();
        let script_txt = doc
            .select(&script_sel)
            .map(text)
            .find(|x| x.contains("var descriptions ="))
            .context(parse_error::MissingElement { selector: "script" })?;

        let descriptions_txt = script_txt
            .split(";\n")
            .next()
            .context(parse_error::MissingElement { selector: "script" })?
            .trim();

        ensure!(
            descriptions_txt.starts_with("var descriptions = {"),
            parse_error::MissingElement { selector: "script" }
        );
        ensure!(
            descriptions_txt.ends_with('}'),
            parse_error::MissingElement { selector: "script" }
        );
        let descriptions_txt = &descriptions_txt[19..];

        let descriptions_str: HashMap<&str, SubInfo> =
            serde_json::from_str(descriptions_txt)?;

        let mut descriptions: HashMap<u64, SubInfo> =
            HashMap::with_capacity(descriptions_str.len());

        for (sid_txt, sub_info) in descriptions_str.into_iter() {
            let sid = sid_txt.parse()?;
            descriptions.insert(sid, sub_info);
        }

        let mut items = vec![];

        let figure_sel =
            Selector::parse("section[id^='gallery-'] > figure").unwrap();
        for figure_elem in doc.select(&figure_sel) {
            let class = attr(figure_elem, "class")?;
            let rating = if class.contains("r-adult") {
                Rating::Adult
            } else if class.contains("r-mature") {
                Rating::Mature
            } else if class.contains("r-general") {
                Rating::General
            } else {
                return Err(ParseError::MissingAttribute {
                    attribute: "class",
                });
            };

            let id_attr = attr(figure_elem, "id")?;
            ensure!(
                id_attr.starts_with("sid-"),
                parse_error::MissingAttribute { attribute: "id" }
            );
            let view_id = id_attr[4..].parse()?;

            let preview_elem = select_first_elem(figure_elem, "img")?;
            let preview_attr = attr(preview_elem, "src")?;
            let preview = url.join(preview_attr)?;

            let sub_info = descriptions.remove(&view_id).unwrap();

            // TODO: sometimes it's a2.facdn.net instead.
            let avatar = url
                .join(&format!(
                    "//a.facdn.net/{}/{}.gif",
                    sub_info.avatar_mtime, sub_info.lower
                ))
                .unwrap();

            items.push(Submission {
                view_id,
                rating,
                preview,
                title: sub_info.title,
                description: sub_info.description,
                artist: MiniUser {
                    name: sub_info.username,
                    slug: sub_info.lower,
                    avatar,
                },
            });
        }

        Ok(Self { items, next, prev })
    }
}
