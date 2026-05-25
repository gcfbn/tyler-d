use anyhow::{Context, Result};
use ocr::ocr_service_client::OcrServiceClient;
use ocr::{OcrRequest, OcrResponse};
use tracing::{debug, error};

pub mod ocr {
    tonic::include_proto!("ocr");
}

pub struct OcrClient {
    client: OcrServiceClient<tonic::transport::Channel>,
}

impl OcrClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = OcrServiceClient::connect(url.to_string())
            .await
            .map_err(|e| {
                error!("Failed to connect to OCR service at {}: {}", url, e);
                e
            })
            .context("Failed to connect to OCR service")?;
        Ok(Self { client })
    }

    pub async fn process_document(&mut self, content: Vec<u8>, file_type: &str) -> Result<OcrResponse> {
        debug!("Calling OCR service for document type: {}", file_type);
        let request = tonic::Request::new(OcrRequest {
            content,
            file_type: file_type.to_string(),
        });

        let response = self.client
            .process_document(request)
            .await
            .map_err(|e| {
                error!("OCR service ProcessDocument call failed: {}", e);
                e
            })
            .context("OCR gRPC call failed")?;

        debug!("OCR service returned successfully");
        Ok(response.into_inner())
    }
}
