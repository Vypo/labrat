use chrono::NaiveDateTime;

use crate::keys::CommentReplyKey;

use scraper::ElementRef;

use snafu::ensure;

use super::{parse_error, MiniUser, ParseError};

use url::Url;

#[derive(Debug, Clone, Copy)]
pub(crate) enum CommentRoot {
    View(u64),
    Journal(u64),
}

#[derive(Debug, Clone)]
pub struct CommentContainer {
    pub(crate) root: CommentRoot,
    pub(crate) comment_id: u64,

    pub(crate) depth: u8,
    pub(crate) comment: Option<Comment>,
}

impl From<CommentContainer> for CommentReplyKey {
    fn from(c: CommentContainer) -> CommentReplyKey {
        From::from(&c)
    }
}

impl From<&CommentContainer> for CommentReplyKey {
    fn from(c: &CommentContainer) -> CommentReplyKey {
        match c.root {
            CommentRoot::View(_) => Self::view_comment(c.comment_id),
            CommentRoot::Journal(_) => Self::journal_comment(c.comment_id),
        }
    }
}

impl CommentContainer {
    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn comment(&self) -> Option<&Comment> {
        self.comment.as_ref()
    }

    fn extract_width(elem: ElementRef) -> Result<u8, ParseError> {
        let style = super::attr(elem, "style")?;
        ensure!(
            style.starts_with("width:"),
            parse_error::InvalidDepth { style }
        );
        ensure!(style.ends_with('%'), parse_error::InvalidDepth { style });
        ensure!(style.len() >= 8, parse_error::InvalidDepth { style });
        ensure!(style.len() <= 10, parse_error::InvalidDepth { style });

        let width_txt = &style[6..style.len() - 1];
        Ok(width_txt.parse()?)
    }

    pub(crate) fn extract(
        url: &Url,
        root: CommentRoot,
        elem: ElementRef,
    ) -> Result<Self, ParseError> {
        let width = Self::extract_width(elem)?;
        let depth = (100 - width) / 3;

        let id_elem =
            super::select_first_elem(elem, "a.comment_anchor[id^='cid:']")?;
        let id_txt = &super::attr(id_elem, "id")?[4..];
        let comment_id: u64 = id_txt.parse()?;

        let text_res = super::select_first_elem(elem, ".comment_text");
        let text = match text_res {
            Ok(t) => crate::html::simplify(url, t),
            Err(ParseError::MissingElement { .. }) => {
                return Ok(CommentContainer {
                    comment: None,
                    comment_id,
                    root,
                    depth,
                });
            }
            Err(e) => return Err(e),
        };

        let parent_res = super::select_first_elem(elem, "a.comment-parent");
        let parent_id = match parent_res {
            Ok(p) => {
                let href = super::attr(p, "href")?;
                ensure!(
                    href.starts_with("#cid:"),
                    parse_error::MissingAttribute { attribute: "href" }
                );
                let parent_id_txt = &href[5..];
                Some(parent_id_txt.parse::<u64>()?)
            }
            Err(ParseError::MissingElement { .. }) => None,
            Err(e) => return Err(e),
        };
        let posted_elem =
            super::select_first_elem(elem, ".comment-date .popup_date")?;
        let posted = super::datetime(posted_elem)?;

        let avatar_elem =
            super::select_first_elem(elem, "img.comment_useravatar")?;
        let avatar = url.join(super::attr(avatar_elem, "src")?)?;

        let slug = super::attr(avatar_elem, "alt")?.to_string();

        let name_elem = super::select_first_elem(elem, ".comment_username h3")?;
        let name = super::text(name_elem);

        Ok(CommentContainer {
            depth,
            root,
            comment_id,
            comment: Some(Comment {
                parent_id,
                text,
                posted,
                commenter: MiniUser { avatar, slug, name },
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub(crate) parent_id: Option<u64>,
    pub(crate) commenter: MiniUser,
    pub(crate) posted: NaiveDateTime,
    pub(crate) text: String,
}

impl Comment {
    pub fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }

    pub fn commenter(&self) -> &MiniUser {
        &self.commenter
    }

    pub fn posted(&self) -> NaiveDateTime {
        self.posted
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
