//! Client HTTP de base.

use crate::error::Error;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::time::Duration;
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
        base_url: impl Into<String>,
        api_key: Option<impl Into<String>>,
    ) -> Result<Self, Error> {
        let mut builder = ClientBuilder::new(base_url);
        if let Some(k) = api_key {
            builder = builder.api_key(k);
        }
        builder.build()
    }

    /// Démarre un builder pour configurer le client (timeout, etc.).
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(base_url)
    }

    /// URL de base configurée.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    pub(crate) fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
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

    pub(crate) async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, Error> {
        let mut req = self.http.post(url).json(body);
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

    pub(crate) async fn get_json_with_query<Q: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        url: Url,
        query: &Q,
    ) -> Result<T, Error> {
        let mut req = self.http.get(url).query(query);
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

    pub(crate) async fn get_text(&self, url: Url) -> Result<String, Error> {
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
        Ok(resp.text().await?)
    }

    pub(crate) async fn patch_no_content<B: Serialize + ?Sized>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<(), Error> {
        let mut req = self.http.patch(url).json(body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(Error::Api { status, detail });
        }
        Ok(())
    }

    pub(crate) async fn delete_no_content(&self, url: Url) -> Result<(), Error> {
        let mut req = self.http.delete(url);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let detail = resp.text().await.unwrap_or_default();
            return Err(Error::Api { status, detail });
        }
        Ok(())
    }
}

/// Builder pour configurer un [`Client`] avec options (timeout, `api_key`).
///
/// # Exemple
/// ```no_run
/// use opengatellm::Client;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), opengatellm::Error> {
/// let client = Client::builder("http://localhost:8000")
///     .api_key("TOKEN")
///     .timeout(Duration::from_secs(30))
///     .build()?;
/// # let _ = client;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    base_url: String,
    api_key: Option<String>,
    timeout: Option<Duration>,
}

impl ClientBuilder {
    /// Initialise un builder avec l'URL de base du gateway.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: None,
            timeout: None,
        }
    }

    /// Bearer token pour l'authentification HTTP.
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Timeout global appliqué à chaque requête HTTP (transport reqwest).
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Construit le [`Client`].
    pub fn build(self) -> Result<Client, Error> {
        let base_url = Url::parse(&self.base_url)?;
        let mut rb = reqwest::Client::builder();
        if let Some(t) = self.timeout {
            rb = rb.timeout(t);
        }
        let http = rb.build()?;
        Ok(Client {
            base_url,
            http,
            api_key: self.api_key,
        })
    }
}
