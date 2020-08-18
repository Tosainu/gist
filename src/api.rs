use std::collections::HashMap;

use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::config::Login;
use crate::error::{Error, ErrorKind, Result};

trait RequestBuilder {
    fn auth(self, login: &Login) -> Self;
}

impl RequestBuilder for reqwest::RequestBuilder {
    fn auth(self, login: &Login) -> Self {
        match login {
            Login::OAuth(token) => self.header(AUTHORIZATION, format!("token {}", token)),
            Login::PersonalAccessToken { username, token } => {
                self.basic_auth(username, Some(token))
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UploadRequest {
    pub files: HashMap<String, FileMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub public: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileMetadata {
    pub content: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GistResponse {
    pub id: String,
    pub html_url: String,
    pub git_pull_url: String,
    pub git_push_url: String,
    pub description: Option<String>,
}

pub type ListResponse = Vec<GistResponse>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateRequest {
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub files: HashMap<String, Option<FileMetadata>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UserResponse {
    pub login: String,
    pub html_url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct VerificationCodeRequest {
    client_id: String,
    scope: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct VerificationCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub interval: u64,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct AccessTokenRequest {
    client_id: String,
    device_code: String,
    grant_type: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum AccessTokenResponse {
    AccessToken { access_token: String },
    Error { error: String },
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn build() -> Result<Self> {
        let b = reqwest::Client::builder().user_agent("reqwest");
        Ok(Client { client: b.build()? })
    }

    pub async fn user(&self, login: &Login) -> Result<UserResponse> {
        let res = self
            .client
            .get("https://api.github.com/user")
            .header(
                ACCEPT,
                HeaderValue::from_static("application/vnd.github.v3+json"),
            )
            .auth(&login)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn upload(&self, login: &Login, req: &UploadRequest) -> Result<GistResponse> {
        let res = self
            .client
            .post("https://api.github.com/gists")
            .auth(&login)
            .json(&req)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json::<GistResponse>().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn update(
        &self,
        login: &Login,
        id: &str,
        req: &UpdateRequest,
    ) -> Result<GistResponse> {
        let res = self
            .client
            .patch(&format!("https://api.github.com/gists/{}", id))
            .auth(&login)
            .json(&req)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json::<GistResponse>().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn list(
        &self,
        login: Option<&Login>,
        username: Option<&str>,
    ) -> Result<ListResponse> {
        let mut builder = if let Some(username) = username {
            self.client
                .get(&format!("https://api.github.com/users/{}/gists", username))
        } else {
            self.client.get("https://api.github.com/gists")
        };

        if let Some(login) = login {
            builder = builder.auth(&login);
        }

        let res = builder.send().await?;
        if res.status().is_success() {
            Ok(res.json::<ListResponse>().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn list_starred(&self, login: &Login) -> Result<ListResponse> {
        let res = self
            .client
            .get("https://api.github.com/gists/starred")
            .auth(&login)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json::<ListResponse>().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn delete(&self, login: &Login, id: &str) -> Result<()> {
        let res = self
            .client
            .delete(&format!("https://api.github.com/gists/{}", id))
            .auth(&login)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn request_verification_code(
        &self,
        client_id: &str,
        scope: &str,
    ) -> Result<VerificationCodeResponse> {
        let req = VerificationCodeRequest {
            client_id: String::from(client_id),
            scope: String::from(scope),
        };
        let res = self
            .client
            .post("https://github.com/login/device/code")
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .json(&req)
            .send()
            .await?;
        if res.status().is_success() {
            Ok(res.json().await?)
        } else {
            Err(Error::new(ErrorKind::ApiWithStatus {
                status: res.status(),
                message: res.text().await?,
            }))
        }
    }

    pub async fn request_access_token(
        &self,
        client_id: &str,
        device_code: &str,
        interval: u64,
    ) -> Result<Login> {
        let req = AccessTokenRequest {
            client_id: String::from(client_id),
            device_code: String::from(device_code),
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_owned(),
        };
        loop {
            time::delay_for(time::Duration::from_secs(interval)).await;

            let res = self
                .client
                .post("https://github.com/login/oauth/access_token")
                .header(ACCEPT, HeaderValue::from_static("application/json"))
                .json(&req)
                .send()
                .await?;
            if res.status().is_success() {
                match res.json::<AccessTokenResponse>().await? {
                    AccessTokenResponse::AccessToken { access_token } => {
                        return Ok(Login::OAuth(access_token))
                    }
                    AccessTokenResponse::Error { error } => match error.as_str() {
                        "authorization_pending" => continue,
                        _ => return Err(Error::new(ErrorKind::Api { message: error })),
                    },
                }
            } else {
                return Err(Error::new(ErrorKind::ApiWithStatus {
                    status: res.status(),
                    message: res.text().await?,
                }));
            }
        }
    }
}
