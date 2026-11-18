use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("could not prepare the node folder")]
    NodeDir(#[source] std::io::Error),
    #[error("could not reserve a local port")]
    Bind(#[source] std::io::Error),
    #[error("could not serve the websocket client api")]
    ClientApi(#[source] std::io::Error),
    #[error("could not build the node configuration")]
    Config(#[source] anyhow::Error),
    #[error("could not build the node")]
    Build(#[source] anyhow::Error),
}
