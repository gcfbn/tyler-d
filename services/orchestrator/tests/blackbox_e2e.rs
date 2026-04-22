use anyhow::Result;
use tscherepacha::orchestrator_client::OrchestratorClient;
use tscherepacha::{IngestRequest, AskRequest};

pub mod tscherepacha {
    tonic::include_proto!("tscherepacha");
}

#[tokio::test]
async fn test_ingest_and_ask_blackbox() -> Result<()> {
    // 1. Connect with a long timeout for slow CPU inference
    let channel = tonic::transport::Channel::from_static("http://localhost:50052")
        .connect_timeout(std::time::Duration::from_secs(30))
        .connect()
        .await?;

    let mut client = OrchestratorClient::new(channel);

    // 2. Action: Ingest a unique fact
    let secret_fact = "Moje tajne haslo do sejfu to: Zolw123";
    let ingest_request = tonic::Request::new(IngestRequest {
        source: Some(tscherepacha::ingest_request::Source::Text(secret_fact.to_string())),
    });

    println!("Ingesting secret fact...");
    let ingest_res = client.ingest(ingest_request).await?.into_inner();
    assert!(ingest_res.success, "Ingestion should be successful");

    // 3. Action: Ask about the secret fact
    let question = "Jakie jest moje tajne haslo do sejfu?";
    let ask_request = tonic::Request::new(AskRequest {
        query: question.to_string(),
    });

    println!("Asking: '{}'", question);
    let ask_res = client.ask(ask_request).await?.into_inner();
    
    println!("Answer received: {}", ask_res.answer);

    // 4. Assertion: Verify the answer contains the secret
    assert!(
        ask_res.answer.to_lowercase().contains("zolw123"),
        "The answer should contain the secret password 'Zolw123'. Got: {}", 
        ask_res.answer
    );

    Ok(())
}
