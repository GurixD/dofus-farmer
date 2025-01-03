use futures::executor::block_on;
use http::Extensions;
use nanoserde::{DeBin, SerBin};
use reqwest::{Request, Response, ResponseBuilderExt};
use reqwest_middleware::{Error, Middleware, Next, Result};
use serde_json::from_str;

use super::api_loader::ApiLoader;

pub struct DofusDbCache {
    version: String,
    directory: String,
}

impl Default for DofusDbCache {
    fn default() -> Self {
        let version: String = from_str(
            &block_on(
                block_on(reqwest::get(format!("{}/version", ApiLoader::BASE_URL)))
                    .unwrap()
                    .text(),
            )
            .unwrap(),
        )
        .unwrap();

        let directory = "http-cacache".to_string();

        Self { version, directory }
    }
}

#[async_trait::async_trait]
impl Middleware for DofusDbCache {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> Result<Response> {
        let key = format!("{}{}", self.version, req.url().as_str());
        let data = cacache::read(&self.directory, &key).await;
        match data {
            Ok(data) => {
                let my_response: MyResponse = DeBin::deserialize_bin(&data[..]).unwrap();
                Ok(http::Response::builder()
                    .status(200)
                    .url(req.url().clone())
                    .body(my_response.body)
                    .unwrap()
                    .into())
            }
            Err(_) => {
                let url = req.url().clone();
                let response = next.run(req, extensions).await?;
                let status = response.status();
                if status == 200 {
                    let body = response.bytes().await?;
                    let data = MyResponse { body: body.into() };
                    cacache::write(&self.directory, &key, data.serialize_bin())
                        .await
                        .unwrap();
                    return Ok(http::Response::builder()
                        .status(status)
                        .url(url)
                        .body(data.body)
                        .unwrap()
                        .into());
                }

                Err(Error::Middleware(anyhow::Error::msg(format!(
                    "wrong status code {status}",
                ))))
            }
        }
    }
}

#[derive(SerBin, DeBin)]
struct MyResponse {
    body: Vec<u8>,
}
