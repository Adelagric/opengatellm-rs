//! Client HTTP de base.

use crate::error::Error;
use serde::de::DeserializeOwned;
use url::Url;

/// Client async pour `OpenGateLLM`.
///
/// Construit une fois, partageable (`Clone` partage le pool de connexions reqwest sous-jacent).
///
/// # Exemple
/// ```no_run
/// use opengatellm::Client;
///
/// # async fn run() -> Result<(), opengatellm::Error> {
/// let client = Client::new("https://albert.api.etalab.gouv.fr", Some("TOKEN"))?;
/// let models = client.models().await?;
/// for m in &models.data {
///     println!("{}", m.id);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    base_url: Url,
    http: reqwest::Client,
    api_key: Option<String>,
}

impl Client {
    /// Crée un client. `base_url` est le `scheme://host[:port]` (sans path),
    /// `api_key` est le bearer token optionnel.
    pub fn new(
        base_url: impl AsRef<str>,
        api_key: Option<impl Into<String>>,
    ) -> Result<Self, Error> {
        let base_url = Url::parse(base_url.as_ref())?;
        let http = reqwest::Client::builder().build()?;
        Ok(Self {
            base_url,
            http,
            api_key: api_key.map(Into::into),
        })
    }

    /// URL de base configurée.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub(crate) fn endpoint(&self, path: &str) -> Result<Url, Error> {
        Ok(self.base_url.join(path)?)
    }

    /// Construit une URL en ajoutant un segment de path (échappé) à un endpoint.
    pub(crate) fn endpoint_with_segment(
        &self,
        base_path: &str,
        segment: &str,
    ) -> Result<Url, Error> {
        let mut url = self.base_url.join(base_path)?;
        url.path_segments_mut()
            .map_err(|()| Error::InvalidUrl("base URL cannot have path segments".into()))?
            .push(segment);
        Ok(url)
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T, Error> {
        let mut req = self.http.get(url);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(Error::Api { status, detail });
        }
        let bytes = resp.bytes().await?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}
