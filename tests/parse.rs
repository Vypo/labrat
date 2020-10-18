use chrono::NaiveDate;

use labrat::keys::{CommentReplyKey, FavKey, SubmissionsKey};
use labrat::resources::header::Header;
use labrat::resources::msg::submissions::Submissions;
use labrat::resources::view::View;
use labrat::resources::{
    FromHtml, ParseError, PreviewSize, Rating, SubmissionKind,
};

use scraper::Html;

use std::convert::TryFrom;

use url::Url;

#[test]
fn view_image() {
    let url = Url::parse("https://www.furaffinity.net/view/38351732/").unwrap();

    let text = include_str!("resources/view/image.html");
    let html = Html::parse_document(text);

    let view = View::from_html(url, &html).unwrap();

    let preview =
        Url::parse("https://t2.facdn.net/38351732@400-1600894374.jpg").unwrap();

    let full = Url::parse(concat!(
        "https://d2.facdn.net/art/candykittycat/1600894374/",
        "1600894374.candykittycat_goat_base001.png"
    ))
    .unwrap();

    let avatar =
        Url::parse("https://a2.facdn.net/1572271060/candykittycat.gif")
            .unwrap();

    let submission = view.submission();
    assert_eq!(submission.preview(PreviewSize::Xxl), preview);
    assert_eq!(submission.rating(), Rating::General);
    assert_eq!(submission.title(), "F2U Goat Base");
    assert_eq!(submission.artist().avatar(), &avatar);
    assert_eq!(submission.artist().slug(), "candykittycat");
    assert_eq!(submission.artist().name(), "candykittycat");
    assert_eq!(submission.kind(), SubmissionKind::Image);

    assert_eq!(view.fullview(), &full);
    assert_eq!(view.download(), &full);

    assert_eq!(view.category(), "All");
    assert_eq!(view.type_(), "All");

    assert_eq!(view.n_views(), 128);
    assert_eq!(view.n_comments(), 16);
    assert_eq!(view.n_favorites(), 25);

    let posted = NaiveDate::from_ymd(2020, 09, 23).and_hms(15, 52, 00);
    assert_eq!(view.posted(), posted);

    assert_eq!(
        view.tags(),
        [
            "adopt",
            "adoptable",
            "adoptables",
            "F2U",
            "free",
            "to",
            "use",
            "goat",
            "base"
        ]
    );

    assert_eq!(view.n_comments(), view.comments().len() as u64);

    let comment_container = &view.comments()[0];
    let key = CommentReplyKey::from(comment_container);
    let exp = CommentReplyKey::try_from(
        "https://www.furaffinity.net/view/38351732/#cid:150154279",
    )
    .unwrap();
    assert_eq!(key, exp);
    assert_eq!(comment_container.depth(), 0);

    let comment = comment_container.comment().unwrap();
    let commented = NaiveDate::from_ymd(2020, 09, 23).and_hms(15, 59, 00);
    assert_eq!(comment.posted(), commented);
    assert_eq!(comment.parent_id(), None);

    let cavatar =
        Url::parse("https://a2.facdn.net/1600406591/luminaria.gif").unwrap();
    assert_eq!(comment.commenter().avatar(), &cavatar);
    assert_eq!(comment.commenter().name(), "Luminaria");
    assert_eq!(comment.commenter().slug(), "luminaria");

    let fav_key = FavKey::try_from(&view).unwrap();
    let exp_fav = FavKey::try_from(
        "https://www.furaffinity.net/fav/38351732/?key=........................................",
    )
    .unwrap();
    assert_eq!(fav_key, exp_fav);
    assert_eq!(view.faved(), Some(false));
}

#[test]
fn view_story() {
    let url = Url::parse("https://www.furaffinity.net/view/37432007/").unwrap();

    let text = include_str!("resources/view/story.html");
    let html = Html::parse_document(text);

    let view = View::from_html(url, &html).unwrap();

    let preview =
        Url::parse("https://t2.facdn.net/37432007@400-1595836340.jpg").unwrap();

    let fullview = Url::parse(concat!(
        "https://d2.facdn.net/art/anubuskiren/stories/1595836340/",
        "1595836340.thumbnail.anubuskiren_hypnoschool_03.rtf.jpg"
    ))
    .unwrap();

    let download = Url::parse(concat!(
        "https://d2.facdn.net/art/anubuskiren/stories/1595836340/",
        "1595836340.anubuskiren_hypnoschool_03.rtf"
    ))
    .unwrap();

    let avatar =
        Url::parse("https://a2.facdn.net/1472366339/anubuskiren.gif").unwrap();

    let submission = view.submission();
    assert_eq!(submission.preview(PreviewSize::Xxl), preview);
    assert_eq!(submission.rating(), Rating::Adult);
    assert_eq!(submission.title(), "Hypno School 03: Incursion");
    assert_eq!(submission.artist().avatar(), &avatar);
    assert_eq!(submission.artist().slug(), "anubuskiren");
    assert_eq!(submission.artist().name(), "AnubusKiren");
    assert_eq!(submission.kind(), SubmissionKind::Text);

    assert_eq!(view.fullview(), &fullview);
    assert_eq!(view.download(), &download);

    assert_eq!(view.category(), "Story");
    assert_eq!(view.type_(), "All");

    assert_eq!(view.n_views(), 829);
    assert_eq!(view.n_comments(), 15);
    assert_eq!(view.n_favorites(), 25);

    let posted = NaiveDate::from_ymd(2020, 07, 27).and_hms(2, 52, 00);
    assert_eq!(view.posted(), posted);

    assert_eq!(
        view.tags(),
        [
            "hypnosis",
            "hypno",
            "hypnotism",
            "mind",
            "control",
            "fox",
            "bunny",
            "hybrid",
            "wolf",
            "angel",
            "demon",
            "incubus",
            "club",
            "school",
            "cat",
            "straight",
            "gay"
        ]
    );

    assert_eq!(view.n_comments(), view.comments().len() as u64);

    let comment_container = &view.comments()[4];
    let id = CommentReplyKey::from(comment_container);
    let exp = CommentReplyKey::try_from(
        "https://furaffinity.net/view/37432007/#cid:148693442",
    )
    .unwrap();
    assert_eq!(id, exp);
    assert_eq!(comment_container.depth(), 0);

    let comment = comment_container.comment().unwrap();
    let commented = NaiveDate::from_ymd(2020, 7, 28).and_hms(12, 26, 00);
    assert_eq!(comment.posted(), commented);
    assert_eq!(comment.parent_id(), None);

    let cavatar =
        Url::parse("https://a2.facdn.net/1555982454/neoxsereki.gif").unwrap();
    assert_eq!(comment.commenter().avatar(), &cavatar);
    assert_eq!(comment.commenter().name(), "Neox_Sereki");
    assert_eq!(comment.commenter().slug(), "neoxsereki");

    let fav_key = FavKey::try_from(&view).unwrap();
    let exp_fav = FavKey::try_from(
        "https://www.furaffinity.net/fav/37432007/?key=........................................",
    )
    .unwrap();
    assert_eq!(fav_key, exp_fav);
    assert_eq!(view.faved(), Some(false));
}

#[test]
fn view_flash() {
    let url = Url::parse("https://www.furaffinity.net/view/10801070/").unwrap();

    let text = include_str!("resources/view/flash.html");
    let html = Html::parse_document(text);

    let view = View::from_html(url, &html).unwrap();

    let preview =
        Url::parse("https://t.facdn.net/10801070@200-1494747184.jpg").unwrap();

    let full = Url::parse(concat!(
        "https://d2.facdn.net/art/jasonafex/1494747184/",
        "1370770387.jasonafex_severus_coil_hypno_stuffing.swf"
    ))
    .unwrap();

    let avatar =
        Url::parse("https://a2.facdn.net/1543350598/jasonafex.gif").unwrap();

    let submission = view.submission();
    assert_eq!(submission.preview(PreviewSize::M), preview);
    assert_eq!(submission.rating(), Rating::Adult);
    assert_eq!(submission.title(), "Hypno Stuffing (Animated)");
    assert_eq!(submission.artist().avatar(), &avatar);
    assert_eq!(submission.artist().slug(), "jasonafex");
    assert_eq!(submission.artist().name(), "Jasonafex");
    assert_eq!(submission.kind(), SubmissionKind::Flash);

    assert_eq!(view.fullview(), &full);
    assert_eq!(view.download(), &full);

    assert_eq!(view.category(), "Flash");
    assert_eq!(view.type_(), "General Furry Art");

    assert_eq!(view.n_views(), 88524);
    assert_eq!(view.n_comments(), 76);
    assert_eq!(view.n_favorites(), 1860);

    let posted = NaiveDate::from_ymd(2013, 06, 09).and_hms(4, 33, 00);
    assert_eq!(view.posted(), posted);

    assert_eq!(
        view.tags(),
        [
            "Dragon",
            "Viper",
            "Kobra",
            "Snake",
            "Reptile",
            "anal",
            "hypno",
            "hypnosis",
            "brainwash",
            "mind",
            "control",
            "cum",
            "Severus",
            "Coil",
            "Jasonafex",
            "flash",
            "animation",
            "1800",
            "contacts",
            "you",
            "cant",
            "have",
            "my",
            "brand"
        ]
    );

    assert_eq!(view.n_comments(), view.comments().len() as u64);

    let comment_container = &view.comments()[6];
    let id = CommentReplyKey::from(comment_container);
    let exp = CommentReplyKey::try_from(
        "https://www.furaffinity.net/view/10801070/#cid:70791506",
    )
    .unwrap();
    assert_eq!(id, exp);
    assert_eq!(comment_container.depth(), 2);

    let comment = comment_container.comment().unwrap();
    let commented = NaiveDate::from_ymd(2013, 6, 16).and_hms(0, 31, 00);
    assert_eq!(comment.posted(), commented);
    assert_eq!(comment.parent_id(), Some(70788912));

    let cavatar =
        Url::parse("https://a2.facdn.net/1468877932/matrixg.gif").unwrap();
    assert_eq!(comment.commenter().avatar(), &cavatar);
    assert_eq!(comment.commenter().name(), "Matrixg");
    assert_eq!(comment.commenter().slug(), "matrixg");

    let fav_key = FavKey::try_from(&view).unwrap();
    let exp_fav = FavKey::try_from(
        "https://www.furaffinity.net/fav/10801070/?key=........................................",
    )
    .unwrap();
    assert_eq!(fav_key, exp_fav);
    assert_eq!(view.faved(), Some(false));
}

#[test]
fn view_music() {
    let url = Url::parse("https://www.furaffinity.net/view/34229773/").unwrap();

    let text = include_str!("resources/view/music.html");
    let html = Html::parse_document(text);

    let view = View::from_html(url, &html).unwrap();

    let preview =
        Url::parse("https://t.facdn.net/34229773@400-1576432093.jpg").unwrap();

    let fullview = Url::parse(concat!(
        "https://d.facdn.net/art/twelvetables/music/1576432093/",
        "1576432093.thumbnail.twelvetables_hypno_pet_mop.mp3.jpg"
    ))
    .unwrap();

    let download = Url::parse(concat!(
        "https://d.facdn.net/art/twelvetables/music/1576432093/",
        "1576432093.twelvetables_hypno_pet_mop.mp3"
    ))
    .unwrap();

    let avatar =
        Url::parse("https://a.facdn.net/1471329951/twelvetables.gif").unwrap();

    let submission = view.submission();
    assert_eq!(submission.preview(PreviewSize::Xxl), preview);
    assert_eq!(submission.rating(), Rating::Adult);
    assert_eq!(
        submission.title(),
        "Real Hypnosis!  Hypno Pet 2: Mind of a Pet"
    );
    assert_eq!(submission.artist().avatar(), &avatar);
    assert_eq!(submission.artist().slug(), "twelvetables");
    assert_eq!(submission.artist().name(), "Twelvetables");
    assert_eq!(submission.kind(), SubmissionKind::Audio);

    assert_eq!(view.fullview(), &fullview);
    assert_eq!(view.download(), &download);

    assert_eq!(view.category(), "Music");
    assert_eq!(view.type_(), "Fetish Other");

    assert_eq!(view.n_views(), 1810);
    assert_eq!(view.n_comments(), 22);
    assert_eq!(view.n_favorites(), 51);

    let posted = NaiveDate::from_ymd(2019, 12, 15).and_hms(12, 48, 00);
    assert_eq!(view.posted(), posted);

    assert_eq!(
        view.tags(),
        [
            "Hypnosis",
            "pet",
            "play",
            "hypno",
            "brainwash",
            "conditioning",
            "horny",
            "hypnotize",
            "cole",
            "arctic",
            "fox",
            "sexy",
            "obedience"
        ]
    );

    let fav_key = FavKey::try_from(&view).unwrap();
    let exp_fav = FavKey::try_from(
        "https://www.furaffinity.net/fav/34229773/?key=........................................",
    )
    .unwrap();
    assert_eq!(fav_key, exp_fav);
    assert_eq!(view.faved(), Some(false));
}

#[test]
fn view_nsfw() {
    let url = Url::parse("https://www.furaffinity.net/view/38375319/").unwrap();

    let text = include_str!("resources/view/nsfw.html");
    let html = Html::parse_document(text);

    let error = View::from_html(url, &html).unwrap_err();

    match error {
        ParseError::Nsfw => (),
        _ => panic!("expected Nsfw error"),
    }
}

#[test]
fn view_header() {
    let url = Url::parse("https://www.furaffinity.net/view/34229773/").unwrap();

    let text = include_str!("resources/view/music.html");
    let html = Html::parse_document(text);

    let header = Header::from_html(url, &html).unwrap();

    assert_eq!(header.me().name(), "aFakeUser");
    assert_eq!(header.me().slug(), "aFakeUser");

    let avatar =
        Url::parse("https://a.facdn.net/1424255659/aFakeUser.gif").unwrap();
    assert_eq!(header.me().avatar(), &avatar);

    let notifs = header.notifications();
    assert_eq!(notifs.notes, 0);
    assert_eq!(notifs.journals, 7577);
    assert_eq!(notifs.trouble_tickets, 0);
    assert_eq!(notifs.submissions, 357);
    assert_eq!(notifs.comments, 0);
    assert_eq!(notifs.watches, 0);
    assert_eq!(notifs.favorites, 0);
}

#[test]
fn msg_submissions_next() {
    let url =
        Url::parse("https://www.furaffinity.net/msg/submissions/").unwrap();

    let text = include_str!("resources/msg/submissions/next.html");
    let html = Html::parse_document(text);

    let page = Submissions::from_html(url, &html).unwrap();

    let next_url = Url::parse(
        "https://www.furaffinity.net/msg/submissions/new~12345678@72/",
    )
    .unwrap();
    let next = SubmissionsKey::try_from(next_url).unwrap();

    assert_eq!(page.next(), Some(&next));
    assert_eq!(page.prev(), None);
}

#[test]
fn msg_submissions_prev() {
    let url = Url::parse(
        "https://www.furaffinity.net/msg/submissions/new~12345678@72/",
    )
    .unwrap();

    let text = include_str!("resources/msg/submissions/prev.html");
    let html = Html::parse_document(text);

    let page = Submissions::from_html(url, &html).unwrap();

    let prev_url = Url::parse(
        "https://www.furaffinity.net/msg/submissions/new~12345678@72/",
    )
    .unwrap();
    let prev = SubmissionsKey::try_from(prev_url).unwrap();

    assert_eq!(page.prev(), Some(&prev));
    assert_eq!(page.next(), None);
}
