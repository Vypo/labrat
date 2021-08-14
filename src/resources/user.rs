use chrono::NaiveDateTime;

use super::{Rating, SubmissionKind, MiniUser};

use url::Url;

#[derive(Debug, Clone)]
pub struct UserJournal {
    journal_id: u64,
    title: String,

    header: Option<String>,
    footer: Option<String>,
    content: String,

    posted: NaiveDateTime,

    n_comments: u64,
}

#[derive(Debug, Clone)]
pub struct MiniSubmission {
    view_id: u64,
    created: u64,
    cdn: Url,
    rating: Rating,
    title: String,
    kind: SubmissionKind,
    artist: MiniUser,
}

#[derive(Debug, Clone)]
pub struct User {
    avatar: Url,
    name: String,
    slug: String,

    profile: String,

    n_views: u64,
    n_submissions: u64,
    n_favorites: u64,
    n_comments_earned: u64,
    n_comments_made: u64,
    n_journals: u64,

    featured_submission: MiniSubmission,

    latest_submissions: Vec<MiniSubmission>,
    latest_favorites: Vec<MiniSubmission>,
    journal: UserJournal,
}
