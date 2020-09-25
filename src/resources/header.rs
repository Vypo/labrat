use scraper::{ElementRef, Html, Selector};

use snafu::{ensure, OptionExt};

use super::{parse_error, FromHtml, MiniUser, ParseError};

use url::Url;

#[derive(Debug, Clone)]
pub struct Header {
    me: MiniUser,
    notifications: Notifications,
}

impl Header {
    pub fn me(&self) -> &MiniUser {
        &self.me
    }

    pub fn notifications(&self) -> &Notifications {
        &self.notifications
    }
}

impl FromHtml for Header {
    fn from_html(url: Url, html: &Html) -> Result<Self, ParseError> {
        let avatar_elem =
            super::select_first(html, "img.loggedin_user_avatar")?;
        let avatar_txt = super::attr(avatar_elem, "src")?;
        let avatar = url.join(avatar_txt)?;
        let name = super::attr(avatar_elem, "alt")?.to_string();

        let slug_node =
            avatar_elem.parent().context(parse_error::MissingElement {
                selector: "img.loggedin_user_avatar < .",
            })?;
        let slug_elem = ElementRef::wrap(slug_node).unwrap();
        let slug_txt = super::attr(slug_elem, "href")?;
        ensure!(slug_txt.starts_with("/user/"), parse_error::IncorrectUrl);
        ensure!(slug_txt.ends_with('/'), parse_error::IncorrectUrl);
        let slug = slug_txt[6..slug_txt.len() - 1].to_string();

        let notifications = Notifications::from_html(url, html)?;

        Ok(Self {
            notifications,
            me: MiniUser { avatar, name, slug },
        })
    }
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Notifications {
    pub submissions: u64,
    pub journals: u64,
    pub watches: u64,
    pub comments: u64,
    pub favorites: u64,
    pub trouble_tickets: u64,
    pub notes: u64,
}

impl Notifications {
    fn suffix(suffix: &str, text: &str) -> Option<u64> {
        if !text.ends_with(suffix) {
            return None;
        }

        let num_text = text[..text.len() - suffix.len()].trim();

        num_text.parse().ok()
    }
}

impl FromHtml for Notifications {
    fn from_html(_: Url, html: &Html) -> Result<Self, ParseError> {
        let bar = super::select_first(html, "#ddmenu .message-bar-desktop")?;

        let selector = Selector::parse("a.notification-container").unwrap();

        let mut n = Notifications {
            submissions: 0,
            journals: 0,
            watches: 0,
            comments: 0,
            favorites: 0,
            trouble_tickets: 0,
            notes: 0,
        };

        for elem in bar.select(&selector) {
            let text = super::text(elem);

            if let Some(tt) = Self::suffix("TT", &text) {
                n.trouble_tickets += tt;
            } else if let Some(s) = Self::suffix("S", &text) {
                n.submissions += s;
            } else if let Some(w) = Self::suffix("W", &text) {
                n.watches += w;
            } else if let Some(c) = Self::suffix("C", &text) {
                n.comments += c;
            } else if let Some(f) = Self::suffix("F", &text) {
                n.favorites += f;
            } else if let Some(notes) = Self::suffix("N", &text) {
                n.notes += notes;
            } else if let Some(j) = Self::suffix("J", &text) {
                n.journals += j;
            }
        }

        Ok(n)
    }
}
