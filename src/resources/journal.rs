use chrono::NaiveDateTime;

use crate::html::simplify;
use crate::keys::{CommentReplyKey, JournalKey};

use scraper::{Html, Selector};

use snafu::{ensure, OptionExt};

use super::comment::{CommentContainer, CommentRoot};
use super::{
    parse_error, select_first, select_first_elem, FromHtml, MiniUser,
    ParseError,
};

use url::Url;

#[derive(Debug, Clone)]
pub struct Journal {
    journal_id: u64,
    title: String,
    author: MiniUser,

    header: Option<String>,
    footer: Option<String>,
    content: String,

    posted: NaiveDateTime,

    n_comments: u64,

    comments: Vec<CommentContainer>,
}

impl Journal {
    pub fn journal_id(&self) -> u64 {
        self.journal_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author(&self) -> &MiniUser {
        &self.author
    }

    pub fn header(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub fn footer(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn posted(&self) -> NaiveDateTime {
        self.posted
    }

    pub fn n_comments(&self) -> u64 {
        self.n_comments
    }

    pub fn comments(&self) -> &[CommentContainer] {
        &self.comments
    }
}

impl FromHtml for Journal {
    fn from_html(url: Url, doc: &Html) -> Result<Self, ParseError> {
        let mut segments =
            url.path_segments().context(parse_error::IncorrectUrl)?;
        ensure!(
            segments.next() == Some("journal"),
            parse_error::IncorrectUrl
        );
        let journal_id_txt =
            segments.next().context(parse_error::IncorrectUrl)?;
        let journal_id = journal_id_txt.parse()?;

        let j = select_first(doc, ".journal-item")?;

        let header = match select_first_elem(j, ".journal-header") {
            Ok(h) => Some(simplify(&url, h)),
            Err(ParseError::MissingElement { .. }) => None,
            Err(e) => return Err(e),
        };

        let footer = match select_first_elem(j, ".journal-footer") {
            Ok(f) => Some(simplify(&url, f)),
            Err(ParseError::MissingElement { .. }) => None,
            Err(e) => return Err(e),
        };

        let content_elem = select_first_elem(j, ".journal-content")?;
        let content = simplify(&url, content_elem);

        let title_elem = select_first(doc, "h2.journal-title")?;
        let title = super::text(title_elem);

        let posted_elem =
            select_first(doc, "h2.journal-title + div .popup_date")?;
        let posted = super::datetime(posted_elem)?;

        let username_sel = "#user-profile .username h2";
        let username_elem = select_first(doc, username_sel)?;
        let username_txt = super::text(username_elem);
        let username_txt = username_txt.trim();
        ensure!(
            username_txt.starts_with('~'),
            parse_error::MissingElement {
                selector: username_sel
            }
        );
        let username = &username_txt[1..];

        let slug_elem =
            select_first(doc, "#user-profile .user-nav a[href^='/user/']")?;
        let slug_attr = &super::attr(slug_elem, "href")?[6..];
        let slug = if let Some(stripped) = slug_attr.strip_suffix('/') {
            stripped
        } else {
            slug_attr
        };

        let avatar_elem =
            select_first(doc, "#user-profile img.user-nav-avatar")?;
        let avatar_txt = super::attr(avatar_elem, "src")?;
        let avatar = url.join(avatar_txt)?;

        let n_comments_elem =
            select_first(doc, ".journal-body-theme + div.section-footer span")?;
        let n_comments = super::number(n_comments_elem)?;

        let comment_root = CommentRoot::Journal(journal_id);

        let comment_sel =
            Selector::parse("#comments-journal .comment_container").unwrap();
        let comments = doc
            .select(&comment_sel)
            .map(|c| CommentContainer::extract(&url, comment_root, c))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            author: MiniUser {
                name: username.to_string(),
                slug: slug.to_string(),
                avatar,
            },
            journal_id,
            content,
            title,
            posted,
            header,
            footer,

            n_comments,
            comments,
        })
    }
}

impl From<&Journal> for CommentReplyKey {
    fn from(v: &Journal) -> Self {
        CommentReplyKey::journal(v.journal_id)
    }
}

impl From<Journal> for CommentReplyKey {
    fn from(v: Journal) -> Self {
        From::from(&v)
    }
}

impl From<&Journal> for JournalKey {
    fn from(v: &Journal) -> Self {
        Self {
            journal_id: v.journal_id,
        }
    }
}

impl From<Journal> for JournalKey {
    fn from(v: Journal) -> Self {
        Self {
            journal_id: v.journal_id,
        }
    }
}
