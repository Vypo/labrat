use chrono::NaiveDateTime;

use crate::keys::{CommentReplyKey, JournalKey, ViewKey};
use crate::resources::comment::CommentRoot;
use crate::resources::{
    attr, datetime, parse_error, select_first_elem, text, FromHtml, MiniUser,
    ParseError,
};

use scraper::{ElementRef, Html, Selector};

use snafu::{ensure, OptionExt};

use url::Url;

#[derive(Debug, Clone)]
pub struct MiniComment {
    root: CommentRoot,
    title: String,
    comment_id: u64,
    author: MiniUser,
    posted: NaiveDateTime,
}

impl From<MiniComment> for CommentReplyKey {
    fn from(mc: MiniComment) -> CommentReplyKey {
        CommentReplyKey::from(&mc)
    }
}

impl From<&MiniComment> for CommentReplyKey {
    fn from(mc: &MiniComment) -> CommentReplyKey {
        match mc.root {
            CommentRoot::View(_) => {
                CommentReplyKey::view_comment(mc.comment_id)
            }
            CommentRoot::Journal(_) => {
                CommentReplyKey::journal_comment(mc.comment_id)
            }
        }
    }
}

impl MiniComment {
    pub fn as_view_key(&self) -> Option<ViewKey> {
        match self.root {
            CommentRoot::View(id) => Some(ViewKey { view_id: id }),
            _ => None,
        }
    }

    pub fn as_journal_key(&self) -> Option<JournalKey> {
        match self.root {
            CommentRoot::Journal(id) => Some(JournalKey { journal_id: id }),
            _ => None,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author(&self) -> &MiniUser {
        &self.author
    }
}

// TODO: impl From<MiniComment> for Option<ViewKey> ??
// TODO: impl From<MiniComment> for Option<JournalKey> ??

#[derive(Debug, Clone)]
pub struct CommentMsg {
    comment_id: u64,
    is_journal: bool,
    comment: Option<MiniComment>,
}

impl CommentMsg {
    fn extract(url: &Url, elem: ElementRef) -> Result<Self, ParseError> {
        let comment_id_elem =
            select_first_elem(elem, "input[name^='comments-']")?;
        let is_journal = attr(comment_id_elem, "name")?.contains("journals");
        let comment_id_txt = attr(comment_id_elem, "value")?;
        let comment_id = comment_id_txt.parse()?;

        let slug_elem = match select_first_elem(elem, "a[href^='/user/']") {
            Ok(e) => e,
            Err(ParseError::MissingElement { .. }) => {
                return Ok(Self {
                    comment_id,
                    is_journal,
                    comment: None,
                })
            }
            Err(e) => return Err(e),
        };
        let slug_attr = "href";
        let mut slug_txt = attr(slug_elem, slug_attr)?;
        ensure!(
            slug_txt.starts_with("/user/"),
            parse_error::MissingAttribute {
                attribute: slug_attr
            },
        );
        if slug_txt.ends_with('/') {
            slug_txt = &slug_txt[..slug_txt.len() - 1];
        }
        let slug = slug_txt[6..].to_string();
        let name = text(slug_elem);

        let posted_elem = select_first_elem(elem, ".popup_date")?;
        let posted = datetime(posted_elem)?;

        let root_elem = select_first_elem(elem, "a[href*='#cid:']")?;
        let root_href = attr(root_elem, "href")?;
        let root_url = url.join(root_href)?;

        let fragment =
            root_url.fragment().context(parse_error::IncorrectUrl)?;
        if !fragment.starts_with("cid:") {
            return Err(ParseError::IncorrectUrl);
        }

        let root_id = fragment[4..].parse()?;

        let root = if is_journal {
            CommentRoot::Journal(root_id)
        } else {
            CommentRoot::View(root_id)
        };

        let title = text(root_elem);

        Ok(Self {
            comment_id,
            is_journal,
            comment: Some(MiniComment {
                author: MiniUser::without_avatar(name, slug),
                title,
                root,
                comment_id,
                posted,
            }),
        })
    }

    pub fn comment(&self) -> Option<&MiniComment> {
        self.comment.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct MiniJournal {
    author: MiniUser,
    posted: NaiveDateTime,
    title: String,
    journal_id: u64,
}

impl MiniJournal {
    pub fn posted(&self) -> NaiveDateTime {
        self.posted
    }

    pub fn author(&self) -> &MiniUser {
        &self.author
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    fn extract(_: &Url, elem: ElementRef) -> Result<Self, ParseError> {
        let journal_id_elem =
            select_first_elem(elem, "input[name='journals[]']")?;
        let journal_id_txt = attr(journal_id_elem, "value")?;
        let journal_id = journal_id_txt.parse()?;

        let slug_elem = select_first_elem(elem, "a[href^='/user/']")?;
        let slug_attr = "href";
        let mut slug_txt = attr(slug_elem, slug_attr)?;
        ensure!(
            slug_txt.starts_with("/user/"),
            parse_error::MissingAttribute {
                attribute: slug_attr
            },
        );
        if slug_txt.ends_with('/') {
            slug_txt = &slug_txt[..slug_txt.len() - 1];
        }
        let slug = slug_txt[6..].to_string();
        let name = text(slug_elem);

        let posted_elem = select_first_elem(elem, ".popup_date")?;
        let posted = datetime(posted_elem)?;

        let root_elem = select_first_elem(elem, "a[href^='/journal/']")?;
        let title = text(root_elem);

        Ok(Self {
            journal_id,
            author: MiniUser::without_avatar(name, slug),
            title,
            posted,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MiniShout {
    author: MiniUser,
    posted: NaiveDateTime,
}

impl MiniShout {
    pub fn author(&self) -> &MiniUser {
        &self.author
    }

    pub fn posted(&self) -> NaiveDateTime {
        self.posted
    }
}

#[derive(Debug, Clone)]
pub struct ShoutMsg {
    shout_id: u64,
    shout: Option<MiniShout>,
}

impl ShoutMsg {
    pub fn shout(&self) -> Option<&MiniShout> {
        self.shout.as_ref()
    }

    fn extract(_: &Url, elem: ElementRef) -> Result<Self, ParseError> {
        // TODO: Include link to user page?

        let shout_id_elem = select_first_elem(elem, "input[name='shouts[]']")?;
        let shout_id_txt = attr(shout_id_elem, "value")?;
        let shout_id = shout_id_txt.parse()?;

        let slug_elem = match select_first_elem(elem, "a[href^='/user/']") {
            Ok(e) => e,
            Err(ParseError::MissingElement { .. }) => {
                return Ok(Self {
                    shout_id,
                    shout: None,
                })
            }
            Err(e) => return Err(e),
        };
        let slug_attr = "href";
        let mut slug_txt = attr(slug_elem, slug_attr)?;
        ensure!(
            slug_txt.starts_with("/user/"),
            parse_error::MissingAttribute {
                attribute: slug_attr
            },
        );
        if slug_txt.ends_with('/') {
            slug_txt = &slug_txt[..slug_txt.len() - 1];
        }
        let slug = slug_txt[6..].to_string();
        let name = text(slug_elem);

        let posted_elem = select_first_elem(elem, ".popup_date")?;
        let posted = datetime(posted_elem)?;

        Ok(Self {
            shout_id,
            shout: Some(MiniShout {
                author: MiniUser::without_avatar(name, slug),
                posted,
            }),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Watch {
    user: MiniUser,
    when: NaiveDateTime,
}

impl Watch {
    pub fn user(&self) -> &MiniUser {
        &self.user
    }

    pub fn when(&self) -> NaiveDateTime {
        self.when
    }
}

#[derive(Debug, Clone)]
pub struct WatchMsg {
    watch_id: u64,
    watch: Option<Watch>,
}

impl WatchMsg {
    fn extract(url: &Url, elem: ElementRef) -> Result<Self, ParseError> {
        let watch_id_elem = select_first_elem(elem, "input[name='watches[]']")?;
        let watch_id_txt = attr(watch_id_elem, "value")?;
        let watch_id = watch_id_txt.parse()?;

        let avatar_img = select_first_elem(elem, ".avatar img")?;
        let avatar_a = match select_first_elem(elem, ".avatar a") {
            Ok(a) => a,
            Err(ParseError::MissingElement { .. }) => {
                return Ok(Self {
                    watch_id,
                    watch: None,
                });
            }
            Err(e) => return Err(e),
        };

        let slug_attr = "href";
        let mut slug_txt = attr(avatar_a, slug_attr)?;
        ensure!(
            slug_txt.starts_with("/user/"),
            parse_error::MissingAttribute {
                attribute: slug_attr
            },
        );
        if slug_txt.ends_with('/') {
            slug_txt = &slug_txt[..slug_txt.len() - 1];
        }
        let slug = slug_txt[6..].to_string();

        let avatar_src = attr(avatar_img, "src")?;
        let avatar = url.join(avatar_src)?;

        let when_elem = select_first_elem(elem, ".info .popup_date")?;
        let when = datetime(when_elem)?;

        let name_elem = select_first_elem(elem, ".info span:first-child")?;
        let name = text(name_elem);

        Ok(Self {
            watch_id,
            watch: Some(Watch {
                when,
                user: MiniUser { avatar, slug, name },
            }),
        })
    }

    pub fn watch(&self) -> Option<&Watch> {
        self.watch.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Favorite {
    favorite_id: u64,
    user: MiniUser,
    view_id: u64,
    when: NaiveDateTime,
    title: String,
}

impl Favorite {
    pub fn user(&self) -> &MiniUser {
        &self.user
    }

    pub fn when(&self) -> NaiveDateTime {
        self.when
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    fn extract(_: &Url, elem: ElementRef) -> Result<Self, ParseError> {
        let fav_id_elem = select_first_elem(elem, "input[name='favorites[]']")?;
        let fav_id_txt = attr(fav_id_elem, "value")?;
        let favorite_id = fav_id_txt.parse()?;

        let view_elem = select_first_elem(elem, "a[href^='/view/']")?;
        let view_attr = "href";
        let mut view_txt = attr(view_elem, view_attr)?;
        ensure!(
            view_txt.starts_with("/view/"),
            parse_error::MissingAttribute {
                attribute: view_attr
            },
        );
        if view_txt.ends_with('/') {
            view_txt = &view_txt[..view_txt.len() - 1];
        }
        let view_id = view_txt[6..].parse()?;
        let title_txt = text(view_elem);
        let mut title = title_txt.as_str();
        if title.starts_with('"') {
            title = &title[1..];
        }
        if title.ends_with('"') {
            title = &title[..title.len() - 1];
        }

        let slug_elem = select_first_elem(elem, "a[href^='/user/']")?;
        let slug_attr = "href";
        let mut slug_txt = attr(slug_elem, slug_attr)?;
        ensure!(
            slug_txt.starts_with("/user/"),
            parse_error::MissingAttribute {
                attribute: slug_attr
            },
        );
        if slug_txt.ends_with('/') {
            slug_txt = &slug_txt[..slug_txt.len() - 1];
        }
        let slug = slug_txt[6..].to_string();
        let name = text(slug_elem);

        let when_elem = select_first_elem(elem, ".popup_date")?;
        let when = datetime(when_elem)?;

        Ok(Self {
            user: MiniUser::without_avatar(name, slug),
            title: title.to_string(),
            favorite_id,
            view_id,
            when,
        })
    }
}

#[derive(Debug)]
pub struct Others {
    journals: Vec<MiniJournal>,
    watches: Vec<WatchMsg>,
    comments: Vec<CommentMsg>,
    favorites: Vec<Favorite>,
    shouts: Vec<ShoutMsg>,
}

impl Others {
    pub fn watches(&self) -> &[WatchMsg] {
        &self.watches
    }

    pub fn comments(&self) -> &[CommentMsg] {
        &self.comments
    }

    pub fn shouts(&self) -> &[ShoutMsg] {
        &self.shouts
    }

    pub fn journals(&self) -> &[MiniJournal] {
        &self.journals
    }

    pub fn favorites(&self) -> &[Favorite] {
        &self.favorites
    }
}

impl FromHtml for Others {
    fn from_html(url: Url, doc: &Html) -> Result<Self, ParseError> {
        let mut watches = Vec::new();
        let watches_sel =
            Selector::parse("#messages-watches .message-stream > li").unwrap();
        for watch_elem in doc.select(&watches_sel) {
            watches.push(WatchMsg::extract(&url, watch_elem)?);
        }

        let mut comments = Vec::new();
        let comments_sel = Selector::parse(
            r#"#messages-comments-submission .message-stream > li,
                   #messages-comments-journal .message-stream > li"#,
        )
        .unwrap();
        for comment_elem in doc.select(&comments_sel) {
            comments.push(CommentMsg::extract(&url, comment_elem)?);
        }

        let mut shouts = Vec::new();
        let shouts_sel =
            Selector::parse("#messages-shouts .message-stream > li").unwrap();
        for shout_elem in doc.select(&shouts_sel) {
            shouts.push(ShoutMsg::extract(&url, shout_elem)?);
        }

        let mut journals = Vec::new();
        let journals_sel =
            Selector::parse("#messages-journals .message-stream > li").unwrap();
        for journal_elem in doc.select(&journals_sel) {
            journals.push(MiniJournal::extract(&url, journal_elem)?);
        }

        let mut favorites = Vec::new();
        let favs_sel =
            Selector::parse("#messages-favorites .message-stream > li")
                .unwrap();
        for journal_elem in doc.select(&favs_sel) {
            favorites.push(Favorite::extract(&url, journal_elem)?);
        }

        Ok(Self {
            watches,
            comments,
            shouts,
            journals,
            favorites,
        })
    }
}
