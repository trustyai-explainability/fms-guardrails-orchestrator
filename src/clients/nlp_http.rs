use async_trait::async_trait;
use axum::extract::Extension;
use tracing::{info, instrument};
use hyper::{HeaderMap, StatusCode};
use tracing::{info, instrument};

use super::{
    create_http_client, Client, Error, HttpClient
};
use crate::{
    config::ServiceConfig,
    health::HealthCheckResult,
    pb::{
        caikit::runtime::nlp::{
            nlp_service_client::NlpServiceClient, ServerStreamingTextGenerationTaskRequest,
            TextGenerationTaskRequest, TokenClassificationTaskRequest, TokenizationTaskRequest,
        },
        caikit_data_model::nlp::{
            GeneratedTextResult, GeneratedTextStreamResult, TokenClassificationResults,
            TokenizationResults,
        },
    },
    tracing_utils::trace_context_from_http_response
};

const DEFAULT_PORT: u16 = 8085;
const MODEL_ID_HEADER_NAME: &str = "mm-model-id";

#[cfg_attr(test, faux::create)]
#[derive(Clone)]
pub struct NlpClientHttp {
    client: HttpClient,
    health_client: Option<HttpClient>,
}

#[cfg_attr(test, faux::methods)]
impl NlpClientHttp {
    pub async fn new(config: &ServiceConfig) -> Self {
        let client = create_http_client(DEFAULT_PORT, config);
        let health_client = if let Some(health_config) = health_config {
            Some(create_http_client(DEFAULT_PORT, health_config).await);
        } else {
            None
        };
        Self {
            client,
            health_client,
        }
    }

    #[instrument(skip_all, fields(request.model))]
    pub async tokenization_task_predict(
        &self,
        request: caikit::runtime::nlp::TokenizationTaskRequest,
        headers: HeaderMap,
    ) -> Result<TokenizationResults, Error> {
        let url = self.client.base_url().join("/api/v1/task/tokenization").unwrap();
        let headers = with_traceparent_header(headers);
        let request - request_with_headers(request, headers);
        info!(?request, "sending request to NLP http service");
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => OK(response.json<TokenizationResult>().await?.into()),
            _ => {
                let code = response.status();
                let message = if let Ok(response) = response.json(<NlpHttpError>).await() {
                    response.message
                } else {
                    "unknown error occured".into()
                };
                Err(Error::Http {code, error})
            }
        }
    }

    pub async token_classification_task_predict(
        &self,
        request: TokenClassificationTaskRequest.
        headers: HeaderMap,
    ) -> Result<TokenClassificationResponse, Error> {
        let url = self.client.base_url().join("/api/v1/task/token-classification").unwrap();
        let headers = with_traceparent_header(headers);
        info!(?request, "sending request to NLP http service");
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<TokenClassification>().await?.into()),
            _ => {
                let code = response.status();
                let message = if let Ok(response) = response.json::<NlpHttpError>().await {
                    response.message
                } else {
                    "unknown error occured".into()
                };
                Err(Error::Http { code, message })
            }
        }

    }

    pub async text_generation_task_predict(
        &self,
        request: TextGenerationTaskRequest,
        headers: HeaderMap,
    ) -> Result<TextGenerationResponse, Error> {
        let url = self.client.base_url().join("/api/v1/task/text-generation").unwrap();
        let headers = with_traceparent_header(headers);
        info!(?request, "sending request to NLP http service");
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<TextGenerationResponse>().await?.into()),
            _ => {
                let code = response.status();
                let message = if let Ok(response) = response.json::<NlpHttpError>().await {
                    response.message
                } else {
                    "unknown error occured".into()
                };
                Err(Error::Http { code, message })
            }
        }
    }

    pub async server_streaming_text_generation_task_predict(
        &self,
        request, ServerStreamingTextGenerationTaskRequest,
        headers: HeaderMap,
    ) -> Result<ServerStreamingTextGenerationTaskResponse, Error> {
        let url = self.client.base_url().join("/api/v1/task/streaming-text-generation").unwrap();
        let headers = with_traceparent_header(headers);
        info!(?request, "sending request to NLP http service");
        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&request)
            .send()
            .await?;
        match response.status() {
            StatusCode::OK => Ok(response.json::<TextGenerationResponse>().await?.into()),
            _ => {
                let code = response.status();
                let message = if let Ok(response) = response.json::<NlpHttpError>().await {
                    response.message
                } else {
                    "unknown error occured".into()
                };
                Err(Error::Http { code, message })
            }
        }
    }
}

#[cfg_attr(test, faux::create)]
#[async_trait]
impl Client for NlpClientHttp {
    fn name(&self) -> &str {
        "nlp_http"
    }
    async fn health(&self) -> HealthCheckResult {
        if let Some(health_client) = &self.health_client {
            health_client.health().await
        } else {
            self.client.health().await
        }
    }
}