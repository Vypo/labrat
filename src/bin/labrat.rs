use labrat::client::Client;

use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(context(false))]
    Client {
        source: labrat::client::ClientError,
    },

    Request {
        source: Box<dyn std::error::Error>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = Client::new()?;
    let view = client
        .view("https://www.furaffinity.net/view/38466622/")
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        .context(Request)?;

    println!("{:#?}", view);

    let journal = client
        .journal("https://www.furaffinity.net/journal/6740803/")
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        .context(Request)?;

    println!("{:#?}", journal);

    Ok(())
}
