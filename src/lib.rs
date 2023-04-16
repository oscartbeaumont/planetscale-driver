pub use deserializer::Database;
pub use response::Deserializer;

use anyhow::Result;
use structs::{ExecuteRequest, ExecuteResponse, Session, VitessError};
use utils::to_base64;

mod response;
mod structs;
mod utils;

pub struct PSConnection {
    pub host: String,
    pub auth: String,
    pub session: Option<Session>,
    pub client: reqwest::Client,
}

impl PSConnection {
    /// Creates a new connection
    pub fn new(host: &str, username: &str, password: &str) -> Self {
        Self {
            host: host.into(),
            auth: format!("Basic {}", to_base64(&format!("{}:{}", username, password))),
            session: None,
            client: reqwest::Client::new(),
        }
    }

    /// Executes a SQL query
    pub async fn execute(&mut self, query: &str) -> Result<ExecuteResponse> {
        let url = format!("https://{}/psdb.v1alpha1.Database/Execute", self.host);
        let sql = ExecuteRequest {
            query: query.into(),
            session: self.session.clone(),
        };

        let res: ExecuteResponse = post(self, &url, sql).await?;
        self.session = Some(res.session.clone());

        Ok(res)
    }

    /// Refreshes the session
    pub async fn refresh(&mut self) -> Result<()> {
        let url = format!("https://{}/psdb.v1alpha1.Database/CreateSession", self.host);
        let res: ExecuteResponse = post_wob(self, &url).await?;
        self.session = Some(res.session.clone());

        Ok(())
    }
}

// MAYBE ![CFG] THIS?
async fn post<B, R>(connection: &PSConnection, url: &str, body: B) -> Result<R>
where
    B: serde::Serialize,
    R: serde::de::DeserializeOwned,
{
    let req = connection
        .client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "database-rust/0.1.0")
        .header("Authorization", &connection.auth)
        .body(serde_json::to_string(&body)?);
    let res = req.send().await?;

    if !res.status().is_success() {
        let error: VitessError = serde_json::from_str(&res.text().await?)?;
        anyhow::bail!("Code: \"{}\", message: \"{}\"", error.code, error.message);
    }

    Ok(serde_json::from_str(&res.text().await?)?)
}

async fn post_wob<R>(connection: &PSConnection, url: &str) -> Result<R>
where
    R: serde::de::DeserializeOwned,
{
    let req = connection
        .client
        .post(url)
        .header("Content-Type", "application/json")
        .header("User-Agent", "database-rust/0.1.0")
        .header("Authorization", &connection.auth)
        .body("{}");
    let res = req.send().await?;

    if !res.status().is_success() {
        let error: VitessError = serde_json::from_str(&res.text().await?)?;
        anyhow::bail!("Code: \"{}\", message: \"{}\"", error.code, error.message);
    }

    Ok(serde_json::from_str(&res.text().await?)?)
}
