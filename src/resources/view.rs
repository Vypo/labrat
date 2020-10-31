use chrono::NaiveDateTime;

use crate::html::simplify;
use crate::keys::{CommentReplyKey, FavKey, FromUrlError, ViewKey};

use scraper::{ElementRef, Html, Selector};

use snafu::{ensure, OptionExt};

use std::convert::TryFrom;

use super::{
    parse_error, select_first, FromHtml, MiniUser, ParseError, PreviewSize,
    Rating, Submission, SubmissionKind, UnauthenticatedError,
};

use url::Url;

#[derive(Debug, Clone)]
pub struct View {
    fav_key: Option<FavKey>,
    faved: Option<bool>,

    submission: Submission,
    fullview: Url,
    download: Url,

    category: String,
    type_: String,

    tags: Vec<String>,

    n_views: u64,
    n_comments: u64,
    n_favorites: u64,

    posted: NaiveDateTime,

    comments: Vec<CommentContainer>,
}

impl TryFrom<&View> for FavKey {
    type Error = UnauthenticatedError;

    fn try_from(v: &View) -> Result<Self, Self::Error> {
        v.fav_key.clone().ok_or(UnauthenticatedError)
    }
}

impl TryFrom<View> for FavKey {
    type Error = UnauthenticatedError;

    fn try_from(v: View) -> Result<Self, Self::Error> {
        v.fav_key.ok_or(UnauthenticatedError)
    }
}

impl From<&View> for CommentReplyKey {
    fn from(v: &View) -> Self {
        CommentReplyKey::view(v.submission.view_id)
    }
}

impl From<View> for CommentReplyKey {
    fn from(v: View) -> Self {
        From::from(&v)
    }
}

impl From<&View> for ViewKey {
    fn from(v: &View) -> Self {
        ViewKey {
            view_id: v.submission.view_id,
        }
    }
}

impl From<View> for ViewKey {
    fn from(v: View) -> Self {
        ViewKey {
            view_id: v.submission.view_id,
        }
    }
}

impl From<View> for Submission {
    fn from(v: View) -> Self {
        v.submission
    }
}

impl From<&View> for Submission {
    fn from(v: &View) -> Self {
        v.submission.clone()
    }
}

impl View {
    pub fn submission(&self) -> &Submission {
        &self.submission
    }

    pub fn preview(&self, sz: PreviewSize) -> Url {
        self.submission.preview(sz)
    }

    pub fn fullview(&self) -> &Url {
        &self.fullview
    }

    pub fn download(&self) -> &Url {
        &self.download
    }

    pub fn faved(&self) -> Option<bool> {
        self.faved
    }

    pub fn category(&self) -> &str {
        &self.category
    }

    pub fn type_(&self) -> &str {
        &self.type_
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn n_views(&self) -> u64 {
        self.n_views
    }

    pub fn n_favorites(&self) -> u64 {
        self.n_favorites
    }

    pub fn n_comments(&self) -> u64 {
        self.n_comments
    }

    pub fn posted(&self) -> NaiveDateTime {
        self.posted
    }

    pub fn comments(&self) -> &[CommentContainer] {
        &self.comments
    }

    fn extract_urls_flash(
        url: &Url,
        doc: &Html,
    ) -> Result<(Url, Url), ParseError> {
        let id0 = url.path_segments().unwrap().nth(1).unwrap();

        let embed = select_first(doc, "object#flash_embed")?;
        let fullview_txt = super::attr(embed, "data")?;
        let fullview = url.join(fullview_txt)?;

        let id1 = fullview.path_segments().unwrap().nth(2).unwrap();

        let preview_txt = format!("//t.facdn.net/{}@200-{}.jpg", id0, id1);
        let preview = url.join(&preview_txt)?;

        Ok((preview, fullview))
    }

    fn extract_urls(
        url: &Url,
        subimg: ElementRef,
    ) -> Result<(Url, Url), ParseError> {
        let fullview_txt = super::attr(subimg, "data-fullview-src")?;
        let fullview = url.join(fullview_txt)?;

        let preview_txt = super::attr(subimg, "data-preview-src")?;
        let preview = url.join(preview_txt)?;

        Ok((preview, fullview))
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

    fn extract_comment(
        url: &Url,
        view_id: u64,
        elem: ElementRef,
    ) -> Result<CommentContainer, ParseError> {
        let width = Self::extract_width(elem)?;
        let depth = (100 - width) / 3;

        let id_elem =
            super::select_first_elem(elem, "a.comment_anchor[id^='cid:']")?;
        let id_txt = &super::attr(id_elem, "id")?[4..];
        let comment_id: u64 = id_txt.parse()?;

        let text_res = super::select_first_elem(elem, ".comment_text");
        let text = match text_res {
            Ok(t) => t.inner_html().trim().to_string(),
            Err(ParseError::MissingElement { .. }) => {
                return Ok(CommentContainer {
                    comment: None,
                    comment_id,
                    view_id,
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
            view_id,
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

impl FromHtml for View {
    fn from_html(url: Url, doc: &Html) -> Result<View, ParseError> {
        let res_subimg = select_first(doc, "img#submissionImg");

        let (preview, fullview) = match res_subimg {
            Ok(img) => Self::extract_urls(&url, img)?,
            Err(ParseError::MissingElement { .. }) => {
                if select_first(doc, "#pageid-matureimage-error").is_ok() {
                    return Err(ParseError::Nsfw);
                }
                Self::extract_urls_flash(&url, doc)?
            }
            Err(e) => return Err(e),
        };

        let (cdn, created) = Submission::parse_url(&preview)?;

        let mut segments =
            url.path_segments().context(parse_error::IncorrectUrl)?;
        ensure!(segments.next() == Some("view"), parse_error::IncorrectUrl);
        let view_id_txt = segments.next().context(parse_error::IncorrectUrl)?;
        let view_id = view_id_txt.parse()?;

        let kind_elem = select_first(doc, "#submission_page")?;
        let kind_class = super::attr(kind_elem, "class")?;
        let kind = if kind_class.contains("page-content-type-flash") {
            SubmissionKind::Flash
        } else if kind_class.contains("page-content-type-image") {
            SubmissionKind::Image
        } else if kind_class.contains("page-content-type-text") {
            SubmissionKind::Text
        } else if kind_class.contains("page-content-type-music") {
            SubmissionKind::Audio
        } else {
            return Err(ParseError::MissingAttribute { attribute: "class" });
        };

        let download_elem = select_first(doc, ".download a")?;
        let download_txt = super::attr(download_elem, "href")?;
        let download = url.join(download_txt)?;

        let category_elem =
            select_first(doc, ".submission-sidebar span.category-name")?;
        let category = super::text(category_elem);

        let type_elem =
            select_first(doc, ".submission-sidebar span.type-name")?;
        let type_ = super::text(type_elem);

        let views_elem =
            select_first(doc, ".stats-container .views .font-large")?;
        let n_views = super::number(views_elem)?;

        let comments_elem =
            select_first(doc, ".stats-container .comments .font-large")?;
        let n_comments = super::number(comments_elem)?;

        let favorites_elem =
            select_first(doc, ".stats-container .favorites .font-large")?;
        let n_favorites = super::number(favorites_elem)?;

        let rating_elem = select_first(doc, ".stats-container .rating-box")?;
        let rating: Rating = super::text(rating_elem).parse()?;

        let posted_elem =
            select_first(doc, ".submission-id-container .popup_date")?;
        let posted = super::datetime(posted_elem)?;

        let title_elem = select_first(
            doc,
            ".submission-id-container .submission-title h2 p",
        )?;
        let title = super::text(title_elem);

        let description_elem = select_first(doc, ".submission-description")?;
        let description = simplify(&url, description_elem);

        let avatar_elem = select_first(doc, ".submission-id-avatar > a > img")?;
        let avatar_txt = super::attr(avatar_elem, "src")?;
        let avatar = url.join(avatar_txt)?;

        let artist_elem = select_first(
            doc,
            ".submission-id-sub-container > a[href^='/user/']",
        )?;
        let user_href = super::attr(artist_elem, "href")?;
        let user_slug = user_href[6..user_href.len() - 1].to_string();
        let user_name = super::text(artist_elem);

        let tag_sel = Selector::parse(".submission-sidebar .tags").unwrap();
        let tags = doc.select(&tag_sel).map(super::text).collect();

        let comment_sel =
            Selector::parse("#comments-submission .comment_container").unwrap();
        let comments = doc
            .select(&comment_sel)
            .map(|c| Self::extract_comment(&url, view_id, c))
            .collect::<Result<Vec<_>, _>>()?;

        let fav_res = select_first(doc, ".favorite-nav a[href^='/fav/']");
        let unfav_res = select_first(doc, ".favorite-nav a[href^='/unfav/']");

        let faved;
        let fav_key_href;

        match (fav_res, unfav_res) {
            (Ok(e), Err(_)) => {
                faved = Some(false);
                fav_key_href = Some(super::attr(e, "href")?);
            }
            (Err(_), Ok(e)) => {
                faved = Some(true);
                fav_key_href = Some(super::attr(e, "href")?);
            }
            (Err(_), Err(_)) => {
                faved = None;
                fav_key_href = None;
            }
            (Ok(_), Ok(_)) => panic!("too many fav links!"),
        }

        let fav_key = if let Some(href) = fav_key_href {
            match FavKey::try_from(url.join(href)?) {
                Ok(k) => Some(k),
                Err(FromUrlError::MissingSegment) => {
                    return Err(ParseError::IncorrectUrl)
                }
                Err(FromUrlError::ParseIntError { source }) => {
                    return Err(ParseError::InvalidInteger { source });
                }
            }
        } else {
            None
        };

        Ok(Self {
            faved,
            fav_key,
            submission: Submission {
                kind,
                view_id,
                created,
                cdn,
                rating,
                title,
                description,
                artist: MiniUser {
                    avatar,
                    name: user_name,
                    slug: user_slug,
                },
            },
            fullview,
            download,
            category,
            type_,
            tags,
            n_views,
            n_comments,
            n_favorites,
            posted,
            comments,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommentContainer {
    view_id: u64,
    comment_id: u64,

    depth: u8,
    comment: Option<Comment>,
}

impl From<CommentContainer> for CommentReplyKey {
    fn from(c: CommentContainer) -> CommentReplyKey {
        From::from(&c)
    }
}

impl From<&CommentContainer> for CommentReplyKey {
    fn from(c: &CommentContainer) -> CommentReplyKey {
        Self::view_comment(c.comment_id)
    }
}

impl CommentContainer {
    pub fn depth(&self) -> u8 {
        self.depth
    }

    pub fn comment(&self) -> Option<&Comment> {
        self.comment.as_ref()
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    parent_id: Option<u64>,
    commenter: MiniUser,
    posted: NaiveDateTime,
    text: String,
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
