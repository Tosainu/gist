use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use reqwest::header::{HeaderValue, ACCEPT, AUTHORIZATION};

use crate::config::Login;

trait RequestBuilder {
    fn auth(self, login: &Login) -> Self;
}

impl RequestBuilder for reqwest::blocking::RequestBuilder {
    fn auth(self, login: &Login) -> Self {
        match login {
            Login::OAuth(token) => self.header(AUTHORIZATION, format!("token {}", token)),
            Login::PersonalAccessToken { user, token } => self.basic_auth(user, Some(token)),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GistRequest {
    files: HashMap<String, FileMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    public: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct FileMetadata {
    // TODO: support binary file
    content: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct GistResponse {
    id: String,
    html_url: String,
    git_pull_url: String,
    git_push_url: String,
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
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn build() -> reqwest::Result<Self> {
        let b = reqwest::blocking::Client::builder().user_agent("reqwest");
        Ok(Client { client: b.build()? })
    }

    pub fn user(&self, login: &Login) -> reqwest::Result<UserResponse> {
        let builder = self
            .client
            .get("https://api.github.com/user")
            .header(
                ACCEPT,
                HeaderValue::from_static("application/vnd.github.v3+json"),
            )
            .auth(&login);

        builder.send()?.json()
    }

    pub fn upload(
        &self,
        login: &Login,
        public: bool,
        description: Option<&str>,
        paths: &[PathBuf],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let files = paths
            .iter()
            .map(|p| {
                let mut buf = String::new();
                let mut f = File::open(p)?;
                f.read_to_string(&mut buf)?;

                let filename = p.file_name().unwrap().to_str().unwrap().to_string();
                Ok((filename, FileMetadata { content: buf }))
            })
            .collect::<io::Result<_>>()?;

        let req = GistRequest {
            files,
            description: description.map(String::from),
            public,
        };
        println!("{:#?}", req);

        println!("{}", serde_json::to_string(&req).unwrap());

        let builder = self
            .client
            .post("https://api.github.com/gists")
            .auth(&login)
            .json(&req);

        let res = builder.send()?;
        println!("{:#?}", res);
        println!("{:#?}", res.json::<GistResponse>());

        Ok(())
    }

    pub fn request_verification_code(
        &self,
        client_id: &str,
        scope: &str,
    ) -> reqwest::Result<VerificationCodeResponse> {
        let req = VerificationCodeRequest {
            client_id: String::from(client_id),
            scope: String::from(scope),
        };
        let res = self
            .client
            .post("https://github.com/login/device/code")
            .header(ACCEPT, HeaderValue::from_static("application/json"))
            .json(&req)
            .send()?;

        println!("{:#?}", res);

        res.json()
    }

    pub fn request_access_token(
        &self,
        client_id: &str,
        device_code: &str,
        interval: u64,
    ) -> reqwest::Result<Login> {
        let req = AccessTokenRequest {
            client_id: String::from(client_id),
            device_code: String::from(device_code),
            grant_type: "urn:ietf:params:oauth:grant-type:device_code".to_owned(),
        };
        loop {
            std::thread::sleep(Duration::from_secs(interval));

            let res = self
                .client
                .post("https://github.com/login/oauth/access_token")
                .header(ACCEPT, HeaderValue::from_static("application/json"))
                .json(&req)
                .send()?;

            match res.json::<AccessTokenResponse>()? {
                AccessTokenResponse::AccessToken { access_token } => {
                    return Ok(Login::OAuth(access_token))
                }
                AccessTokenResponse::Error { error } => match error.as_str() {
                    "authorization_pending" => continue,
                    _ => panic!("{}", error), // TODO:
                },
            }
        }
    }
}
