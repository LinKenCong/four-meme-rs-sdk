#![allow(dead_code)]

use four_meme_sdk::{FourMemeSdk, SdkConfig};
use httpmock::{Method, Mock, MockServer};
use serde_json::Value;

pub struct MockFourMemeApi {
    server: MockServer,
}

impl MockFourMemeApi {
    pub fn start() -> Self {
        Self {
            server: MockServer::start(),
        }
    }

    pub fn sdk(&self) -> FourMemeSdk {
        FourMemeSdk::new(SdkConfig::new().with_api_base(self.base_url()))
            .expect("mock SDK config should be valid")
    }

    pub fn get_json(&self, path: &str, body: Value) -> Mock<'_> {
        self.server.mock(|when, then| {
            when.method(Method::GET).path(path);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(body);
        })
    }

    pub fn get_json_with_query(&self, path: &str, query: (&str, String), body: Value) -> Mock<'_> {
        self.server.mock(|when, then| {
            when.method(Method::GET)
                .path(path)
                .query_param(query.0, query.1);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(body);
        })
    }

    pub fn post_json(&self, path: &str, body: Value) -> Mock<'_> {
        self.server.mock(|when, then| {
            when.method(Method::POST).path(path);
            then.status(200)
                .header("content-type", "application/json")
                .json_body(body);
        })
    }

    fn base_url(&self) -> String {
        self.server.base_url()
    }
}
