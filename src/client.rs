mod errors {
    use reqwest::StatusCode;

    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum ClientError {
        #[snafu(context(false))]
        Reqwest { source: reqwest::Error },
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub enum RequestError<E>
    where
        E: 'static + std::error::Error,
    {
        Parse {
            source: crate::resources::ParseError,
        },
        #[snafu(context(false))]
        Reqwest {
            source: reqwest::Error,
        },
        Unsuccessful {
            status: StatusCode,
        },
        KeyError {
            source: E,
        },
    }
}

use crate::keys::{
    CommentReplyKey, FavKey, FromStrError, FromUrlError, SubmissionsKey,
    ViewKey,
};
use crate::resources::header::Header;
use crate::resources::msg::submissions::Submissions;
use crate::resources::view::View;
use crate::resources::{FromHtml, ParseError};

use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
use reqwest::ClientBuilder;

use scraper::Html;

pub use self::errors::{ClientError, RequestError};

use serde::Serialize;

use snafu::{ensure, ResultExt};

use std::convert::{Infallible, TryInto};

use tokio::sync::RwLock;

use url::Url;

impl From<RequestError<Infallible>> for RequestError<FromStrError> {
    fn from(o: RequestError<Infallible>) -> Self {
        match o {
            RequestError::Unsuccessful { status } => {
                RequestError::Unsuccessful { status }
            }
            RequestError::Reqwest { source } => {
                RequestError::Reqwest { source }
            }
            RequestError::Parse { source } => RequestError::Parse { source },
            RequestError::KeyError { .. } => unreachable!(),
        }
    }
}

impl From<RequestError<Infallible>> for RequestError<FromUrlError> {
    fn from(o: RequestError<Infallible>) -> Self {
        match o {
            RequestError::Unsuccessful { status } => {
                RequestError::Unsuccessful { status }
            }
            RequestError::Reqwest { source } => {
                RequestError::Reqwest { source }
            }
            RequestError::Parse { source } => RequestError::Parse { source },
            RequestError::KeyError { .. } => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Response<V> {
    pub header: Option<Header>,
    pub page: V,
}

impl<V> FromHtml for Response<V>
where
    V: FromHtml,
{
    fn from_html(url: Url, html: &Html) -> Result<Self, ParseError> {
        Ok(Self {
            header: Header::from_html(url.clone(), html).ok(),
            page: V::from_html(url, html)?,
        })
    }
}

#[derive(Debug)]
pub struct Client {
    client: RwLock<reqwest::Client>,
}

impl Client {
    const USER_AGENT: &'static str = concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION"),
        " (vypo@fursuits.by)",
    );

    fn builder() -> ClientBuilder {
        ClientBuilder::new()
            .cookie_store(true)
            .user_agent(Self::USER_AGENT)
    }

    pub fn new() -> Result<Self, ClientError> {
        let builder = Self::builder();
        Ok(Self {
            client: RwLock::new(builder.build()?),
        })
    }

    pub fn with_cookies<H>(cookies: H) -> Result<Self, ClientError>
    where
        H: Into<HeaderValue>,
    {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.into());

        let builder = Self::builder().default_headers(headers);

        Ok(Self {
            client: RwLock::new(builder.build()?),
        })
    }

    pub async fn set_cookies<H>(&self, cookies: H) -> Result<(), ClientError>
    where
        H: Into<HeaderValue>,
    {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, cookies.into());

        let mut client = self.client.write().await;
        *client = Self::builder().default_headers(headers).build()?;
        Ok(())
    }

    pub async fn view<K>(
        &self,
        key: K,
    ) -> Result<Response<View>, RequestError<K::Error>>
    where
        K: TryInto<ViewKey>,
        K::Error: 'static + std::error::Error,
    {
        let key = key.try_into().context(errors::KeyError)?;
        let url = Url::from(key);

        let response = self.client.read().await.get(url.clone()).send().await?;

        ensure!(
            response.status().is_success(),
            errors::Unsuccessful {
                status: response.status()
            },
        );

        let text = response.text().await?;
        let html = Html::parse_document(&text);
        Ok(Response::from_html(url, &html).context(errors::Parse)?)
    }

    pub async fn reply<K>(
        &self,
        to: K,
        comment: &str,
    ) -> Result<(), RequestError<K::Error>>
    where
        K: TryInto<CommentReplyKey>,
        K::Error: 'static + std::error::Error,
    {
        #[derive(Serialize)]
        struct Form<'a> {
            reply: &'a str,
            replyto: &'a str,
            action: &'a str,
            send: &'a str,
        }

        let key = to.try_into().context(errors::KeyError)?;
        let url = Url::from(key);

        let form = Form {
            action: "reply",
            reply: comment,
            replyto: "",
            send: "send",
        };

        let response = self
            .client
            .read()
            .await
            .post(url.clone())
            .form(&form)
            .send()
            .await?;

        ensure!(
            response.status().is_success(),
            errors::Unsuccessful {
                status: response.status()
            },
        );

        // TODO: check for errors in the HTML

        Ok(())
    }

    pub async fn fav<K>(&self, view: K) -> Result<(), RequestError<K::Error>>
    where
        K: TryInto<FavKey>,
        K::Error: 'static + std::error::Error,
    {
        self.maybe_fav(view, true).await
    }

    pub async fn unfav<K>(&self, view: K) -> Result<(), RequestError<K::Error>>
    where
        K: TryInto<FavKey>,
        K::Error: 'static + std::error::Error,
    {
        self.maybe_fav(view, false).await
    }

    async fn maybe_fav<K>(
        &self,
        view: K,
        fav: bool,
    ) -> Result<(), RequestError<K::Error>>
    where
        K: TryInto<FavKey>,
        K::Error: 'static + std::error::Error,
    {
        let key = view.try_into().context(errors::KeyError)?;
        let txt = format!("https://www.furaffinity.net/{}", key.suffix(fav));
        let url = Url::parse(&txt).unwrap();

        let response = self.client.read().await.get(url).send().await?;

        ensure!(
            response.status().is_success(),
            errors::Unsuccessful {
                status: response.status()
            },
        );

        // TODO: check for errors in the HTML

        Ok(())
    }

    pub async fn submissions<K>(
        &self,
        key: K,
    ) -> Result<Response<Submissions>, RequestError<K::Error>>
    where
        K: TryInto<SubmissionsKey>,
        K::Error: 'static + std::error::Error,
    {
        let key = key.try_into().context(errors::KeyError)?;
        let url = Url::from(key);

        let response = self.client.read().await.get(url.clone()).send().await?;

        ensure!(
            response.status().is_success(),
            errors::Unsuccessful {
                status: response.status()
            },
        );

        let text = response.text().await?;
        let html = Html::parse_document(&text);
        Ok(Response::from_html(url, &html).context(errors::Parse)?)
    }
}
