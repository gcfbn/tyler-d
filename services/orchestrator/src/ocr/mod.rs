use anyhow::{Context, Result};
use ocr::ocr_service_client::OcrServiceClient;
use ocr::{OcrRequest, OcrResponse};

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
            .context("Failed to connect to OCR service")?;
        Ok(Self { client })
    }

    pub async fn process_document(&mut self, content: Vec<u8>, file_type: &str) -> Result<OcrResponse> {
        let request = tonic::Request::new(OcrRequest {
            content,
            file_type: file_type.to_string(),
        });

        let response = self.client
            .process_document(request)
            .await
            .context("OCR gRPC call failed")?;

        Ok(response.into_inner())
    }
}
