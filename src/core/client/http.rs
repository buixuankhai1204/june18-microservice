use reqwest::Response;
use serde::Serialize;
use std::time::Duration;

use crate::core::configure::app::AppConfig;
use crate::infrastructure::error::AppResult;

pub type HttpClient = reqwest::Client;

pub trait ClientBuilder: Sized {
    fn build_from_config(config: &AppConfig) -> AppResult<Self>;
}

pub trait HttpClientExt: ClientBuilder {
    fn post_request<T: Serialize + ?Sized + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>>;
    fn put_request<T: Serialize + ?Sized + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>>;
    fn delete_request(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>>;
    fn get_request(
        &self,
        url: &str,
    ) -> impl std::future::Future<Output = Result<Response, reqwest::Error>>;
}

impl ClientBuilder for HttpClient {
    fn build_from_config(config: &AppConfig) -> AppResult<Self> {
        Ok(reqwest::Client::builder()
            .timeout(Duration::from_secs(config.http.timeout.as_secs()))
            .build()
            .unwrap())
    }
}

impl HttpClientExt for HttpClient {
    async fn post_request<T: Serialize + ?Sized + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<Response, reqwest::Error> {
        self.post(url).json(body).send().await
    }

    async fn put_request<T: Serialize + ?Sized + Send + Sync>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<Response, reqwest::Error> {
        self.put(url).json(body).send().await
    }

    async fn delete_request(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.delete(url).send().await
    }

    async fn get_request(&self, url: &str) -> Result<Response, reqwest::Error> {
        self.get(url).send().await
    }
}
