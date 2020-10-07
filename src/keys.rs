mod errors {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum FromUrlError {
        MissingSegment,
        #[snafu(context(false))]
        ParseIntError {
            source: std::num::ParseIntError,
        },
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum FromStrError {
        MalformedUrl { source: url::ParseError },
        FromUrl { source: FromUrlError },
    }
}

use crate::resources::msg::submissions::Order;

pub use self::errors::{FromStrError, FromUrlError};

use snafu::{ensure, OptionExt, ResultExt};

use std::convert::{TryFrom, TryInto};

use url::Url;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SubmissionsKey {
    order: Order,
    after: Option<u64>,
}

impl SubmissionsKey {
    pub fn newest() -> Self {
        Self {
            order: Order::Descending,
            after: None,
        }
    }

    pub fn oldest() -> Self {
        Self {
            order: Order::Ascending,
            after: None,
        }
    }
}

impl Default for SubmissionsKey {
    fn default() -> Self {
        Self::oldest()
    }
}

impl TryFrom<Url> for SubmissionsKey {
    type Error = FromUrlError;

    fn try_from(u: Url) -> Result<Self, Self::Error> {
        TryFrom::try_from(&u)
    }
}

impl TryFrom<&Url> for SubmissionsKey {
    type Error = FromUrlError;

    fn try_from(u: &Url) -> Result<Self, Self::Error> {
        let mut path = u.path_segments().context(errors::MissingSegment)?;
        ensure!(path.next() == Some("msg"), errors::MissingSegment);
        ensure!(path.next() == Some("submissions"), errors::MissingSegment);

        let segment = match path.next() {
            Some(t) => t,
            None => return Ok(Self::default()),
        };

        let order_id =
            segment.split('@').next().context(errors::MissingSegment)?;

        let mut parts = order_id.split('~');
        let order_txt = parts.next().context(errors::MissingSegment)?;

        let order = match order_txt {
            "new" => Order::Descending,
            "old" => Order::Ascending,
            _ => return Err(FromUrlError::MissingSegment),
        };

        let after = match parts.next() {
            None => None,
            Some(x) => Some(x.parse()?),
        };

        Ok(Self { order, after })
    }
}

impl From<&SubmissionsKey> for Url {
    fn from(k: &SubmissionsKey) -> Url {
        let after = k.after.map(|id| format!("~{}", id)).unwrap_or_default();
        let text = format!(
            "https://www.furaffinity.net/msg/submissions/{}{}@72/",
            k.order.text(),
            after,
        );
        Url::parse(&text).unwrap()
    }
}

impl From<SubmissionsKey> for Url {
    fn from(k: SubmissionsKey) -> Url {
        From::from(&k)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FavKey {
    view_id: u64,
    key: String,
}

impl FavKey {
    pub(crate) fn suffix(&self, fav: bool) -> String {
        if fav {
            format!("fav/{}/?key={}", self.view_id, self.key)
        } else {
            format!("unfav/{}/?key={}", self.view_id, self.key)
        }
    }
}

impl TryFrom<Url> for FavKey {
    type Error = FromUrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        TryFrom::try_from(&url)
    }
}

impl TryFrom<&Url> for FavKey {
    type Error = FromUrlError;

    fn try_from(url: &Url) -> Result<Self, Self::Error> {
        let mut path = url.path_segments().context(errors::MissingSegment)?;
        let mode = path.next();
        ensure!(
            mode == Some("fav") || mode == Some("unfav"),
            errors::MissingSegment
        );
        let view_id = path.next().context(errors::MissingSegment)?.parse()?;

        for (k, v) in url.query_pairs() {
            if k == "key" {
                return Ok(Self {
                    view_id,
                    key: v.to_string(),
                });
            }
        }

        Err(FromUrlError::MissingSegment)
    }
}

impl TryFrom<&str> for FavKey {
    type Error = FromStrError;

    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(txt).context(errors::MalformedUrl)?;
        url.try_into().context(errors::FromUrl)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum ReplyTo {
    View(u64),
    Journal(u64),

    ViewComment(u64),
    JournalComment(u64),
}

impl From<ReplyTo> for Url {
    fn from(r: ReplyTo) -> Url {
        From::from(&r)
    }
}

impl From<&ReplyTo> for Url {
    fn from(r: &ReplyTo) -> Url {
        let txt = match r {
            ReplyTo::View(v) => {
                format!("https://www.furaffinity.net/view/{}/", v)
            }
            ReplyTo::Journal(j) => {
                format!("https://www.furaffinity.net/journal/{}/", j)
            }
            ReplyTo::ViewComment(cid) => format!(
                "https://www.furaffinity.net/replyto/submission/{}/",
                cid
            ),
            ReplyTo::JournalComment(cid) => {
                format!("https://www.furaffinity.net/replyto/journal/{}/", cid)
            }
        };

        Url::parse(&txt).unwrap()
    }
}

impl ReplyTo {
    fn parse_fragment(url: &Url) -> Option<Result<u64, FromUrlError>> {
        let fragment = url.fragment()?;
        if !fragment.starts_with("cid:") {
            return Some(Err(FromUrlError::MissingSegment));
        }

        match fragment[4..].parse() {
            Err(source) => Some(Err(FromUrlError::ParseIntError { source })),
            Ok(i) => Some(Ok(i)),
        }
    }
}

impl TryFrom<&Url> for ReplyTo {
    type Error = errors::FromUrlError;

    fn try_from(url: &Url) -> Result<Self, Self::Error> {
        let mut path = url.path_segments().context(errors::MissingSegment)?;

        match path.next() {
            None => Err(FromUrlError::MissingSegment),
            Some("journal") => match Self::parse_fragment(url) {
                Some(Err(e)) => Err(e),
                Some(Ok(cid)) => Ok(ReplyTo::JournalComment(cid)),
                None => {
                    let id = path.next().context(errors::MissingSegment)?;
                    Ok(ReplyTo::Journal(id.parse()?))
                }
            },
            Some("view") => match Self::parse_fragment(url) {
                Some(Err(e)) => Err(e),
                Some(Ok(cid)) => Ok(ReplyTo::ViewComment(cid)),
                None => {
                    let id = path.next().context(errors::MissingSegment)?;
                    Ok(ReplyTo::View(id.parse()?))
                }
            },
            Some("replyto") => match path.next() {
                None => Err(FromUrlError::MissingSegment),
                Some(v) => {
                    let cid =
                        path.next().context(errors::MissingSegment)?.parse()?;
                    match v {
                        "journal" => Ok(ReplyTo::JournalComment(cid)),
                        "submission" => Ok(ReplyTo::ViewComment(cid)),
                        _ => Err(FromUrlError::MissingSegment),
                    }
                }
            },
            Some(_) => Err(FromUrlError::MissingSegment),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct CommentReplyKey {
    reply_to: ReplyTo,
}

impl CommentReplyKey {
    pub(crate) fn view(id: u64) -> Self {
        Self {
            reply_to: ReplyTo::View(id),
        }
    }

    pub(crate) fn view_comment(cid: u64) -> Self {
        Self {
            reply_to: ReplyTo::ViewComment(cid),
        }
    }
}

impl TryFrom<&str> for CommentReplyKey {
    type Error = FromStrError;

    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(txt).context(errors::MalformedUrl)?;
        url.try_into().context(errors::FromUrl)
    }
}

impl TryFrom<Url> for CommentReplyKey {
    type Error = FromUrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        TryFrom::try_from(&url)
    }
}

impl TryFrom<&Url> for CommentReplyKey {
    type Error = FromUrlError;

    fn try_from(url: &Url) -> Result<Self, Self::Error> {
        Ok(Self {
            reply_to: url.try_into()?,
        })
    }
}

impl From<&CommentReplyKey> for Url {
    fn from(key: &CommentReplyKey) -> Url {
        Url::from(key.reply_to)
    }
}

impl From<CommentReplyKey> for Url {
    fn from(key: CommentReplyKey) -> Url {
        Url::from(key.reply_to)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ViewKey {
    pub view_id: u64,
}

impl TryFrom<Url> for ViewKey {
    type Error = FromUrlError;

    fn try_from(url: Url) -> Result<Self, Self::Error> {
        TryFrom::try_from(&url)
    }
}

impl TryFrom<&Url> for ViewKey {
    type Error = FromUrlError;

    fn try_from(url: &Url) -> Result<Self, Self::Error> {
        let mut segments =
            url.path_segments().context(errors::MissingSegment)?;

        ensure!(segments.next() == Some("view"), errors::MissingSegment);

        let text = segments.next().context(errors::MissingSegment)?;
        let view_id = text.parse()?;

        Ok(ViewKey { view_id })
    }
}

impl TryFrom<&str> for ViewKey {
    type Error = FromStrError;

    fn try_from(txt: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(txt).context(errors::MalformedUrl)?;
        url.try_into().context(errors::FromUrl)
    }
}

impl From<ViewKey> for Url {
    fn from(key: ViewKey) -> Url {
        let txt = format!("https://www.furaffinity.net/view/{}/", key.view_id);
        Url::parse(&txt).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submissions_key_from_new_id() {
        let url = Url::parse(
            "https://www.furaffinity.net/msg/submissions/new~38549204@48/",
        )
        .unwrap();

        let actual = SubmissionsKey::try_from(url).unwrap();
        let expected = SubmissionsKey {
            order: Order::Descending,
            after: Some(38549204),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn submissions_key_from_newest() {
        let url =
            Url::parse("https://www.furaffinity.net/msg/submissions/new@48/")
                .unwrap();

        let actual = SubmissionsKey::try_from(url).unwrap();
        let expected = SubmissionsKey {
            order: Order::Descending,
            after: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn submissions_key_from_oldest() {
        let url =
            Url::parse("https://www.furaffinity.net/msg/submissions/old@48/")
                .unwrap();

        let actual = SubmissionsKey::try_from(url).unwrap();
        let expected = SubmissionsKey {
            order: Order::Ascending,
            after: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_view_journal() {
        let url = Url::parse(
            "https://www.furaffinity.net/replyto/journal/150332622/",
        )
        .unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::JournalComment(150332622),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_view_replyto() {
        let url = Url::parse(
            "https://www.furaffinity.net/replyto/submission/150332622/",
        )
        .unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::ViewComment(150332622),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_view() {
        let url =
            Url::parse("https://www.furaffinity.net/view/9573919/").unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::View(9573919),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_journal() {
        let url =
            Url::parse("https://www.furaffinity.net/journal/9573919/").unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::Journal(9573919),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_journal_comment() {
        let url = Url::parse(
            "https://www.furaffinity.net/journal/9573919/#cid:57397217",
        )
        .unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::JournalComment(57397217),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn comment_reply_key_from_url_view_comment() {
        let url = Url::parse(
            "https://www.furaffinity.net/view/9573919/#cid:57397217",
        )
        .unwrap();

        let actual = CommentReplyKey::try_from(url).unwrap();
        let expected = CommentReplyKey {
            reply_to: ReplyTo::ViewComment(57397217),
        };

        assert_eq!(actual, expected);
    }
}
