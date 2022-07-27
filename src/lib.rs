//! # Icarus
//!
//! Icarus is a library which provides models and methods to fetch metadata about games

#![warn(missing_docs, unused_import_braces, missing_debug_implementations)]

use util::sha1_hash;

/// Models and methods for fetching metadata for Minecraft
pub mod minecraft;
/// Models and methods for fetching metadata for Minecraft mod loaders
pub mod modded;
// Methods for interacting with maven repositories.
pub(crate) mod util;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
/// An error type representing possible errors when fetching metadata
pub enum Error {
    /// A checksum was failed to validate for a file.
    #[error("Failed to validate file checksum at url {url} with hash {hash}.")]
    ChecksumFailure {
        /// The checksum's hash
        hash: String,
        /// The URL of the file attempted to be downloaded
        url: String,
    },

    /// Maven error.
    #[error("Maven error: {0}")]
    Maven(#[from] util::MavenError),

    /// serde error.
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    /// reqwest error.
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Error when managing async tasks.
    #[error("Error while managing asynchronous tasks.")]
    Join(#[from] tokio::task::JoinError),

    /// Error while parsing input
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone)]
struct Icarus {
    client: reqwest::Client,
}

impl Icarus {
    fn create_default_client() -> Result<reqwest::Client> {
        let client = reqwest::Client::builder()
            .tcp_keepalive(Some(std::time::Duration::from_secs(10)))
            .connect_timeout(std::time::Duration::from_secs(15))
            .build()?;

        Ok(client)
    }

    /// Downloads a file from specified mirrors
    pub async fn download_file_mirrors(
        &self,
        base: &str,
        mirrors: &[&str],
        sha1: Option<&str>,
    ) -> Result<bytes::Bytes> {
        if mirrors.is_empty() {
            return Err(Error::Parse("No mirrors provided!".to_string()));
        }

        for (index, mirror) in mirrors.iter().enumerate() {
            let result = self
                .download_file(&*format!("{}{}", mirror, base), sha1)
                .await;

            if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
                return result;
            }
        }

        unreachable!()
    }

    // Downloads a file with checksum functionality.
    pub async fn download_file(&self, url: &str, sha1: Option<&str>) -> Result<bytes::Bytes> {
        let bytes = self.client.get(url).send().await?.bytes().await?;

        if let Some(sha1) = sha1 {
            if &*sha1_hash(bytes.clone()).await? != sha1 {
                return Err(Error::ChecksumFailure {
                    hash: sha1.to_string(),
                    url: url.to_string(),
                });
            }
        }

        Ok(bytes)
    }
}
