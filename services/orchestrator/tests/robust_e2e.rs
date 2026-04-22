use anyhow::Result;
use tscherepacha::orchestrator_client::OrchestratorClient;
use tscherepacha::{IngestRequest, AskRequest};
use std::time::Duration;
use fake::{Fake, locales::EN};
use fake::faker::name::raw::Name;
use fake::faker::address::raw::CityName;
use fake::faker::company::raw::CatchPhrase;

pub mod tscherepacha {
    tonic::include_proto!("tscherepacha");
}

async fn get_client() -> Result<OrchestratorClient<tonic::transport::Channel>> {
    let channel = tonic::transport::Channel::from_static("http://localhost:50052")
        .connect_timeout(Duration::from_secs(30))
        .connect()
        .await?;
    Ok(OrchestratorClient::new(channel))
}

#[tokio::test]
async fn test_high_volume_random_data() -> Result<()> {
    let mut client = get_client().await?;

    println!("Generating and ingesting 100 random facts (load test)...");
    for i in 0..100 {
        let name: String = Name(EN).fake();
        let city: String = CityName(EN).fake();
        let catch_phrase: String = CatchPhrase(EN).fake(); 
        let fact = format!("Person {} from {} says: {}. (ID: {})", name, city, catch_phrase, i);
        
        client.ingest(tonic::Request::new(IngestRequest {
            source: Some(tscherepacha::ingest_request::Source::Text(fact)),
        })).await?;
    }

    // Needle in a haystack
    let needle = "Mój wujek Hieronim ukrył klucze do piwnicy za starym obrazem z jeleniem w Krakowie.";
    client.ingest(tonic::Request::new(IngestRequest {
        source: Some(tscherepacha::ingest_request::Source::Text(needle.to_string())),
    })).await?;

    let question = "Gdzie Hieronim ukrył klucze?";
    let res = client.ask(tonic::Request::new(AskRequest { query: question.to_string() })).await?.into_inner();
    println!("Answer: {}", res.answer);
    
    assert!(res.answer.to_lowercase().contains("obrazem") || res.answer.to_lowercase().contains("jeleniem"));
    Ok(())
}

#[tokio::test]
async fn test_evolving_information_multistep() -> Result<()> {
    let mut client = get_client().await?;

    let updates = vec![
        "Mój projekt nazywa się 'Alpha' i jest w fazie planowania.",
        "Zmieniłem nazwę projektu na 'Beta'. Wciąż planujemy.",
        "Projekt 'Beta' przeszedł do fazy programowania.",
        "Projekt 'Beta' został właśnie ukończony i wydany!"
    ];

    for update in updates {
        client.ingest(tonic::Request::new(IngestRequest {
            source: Some(tscherepacha::ingest_request::Source::Text(update.to_string())),
        })).await?;
    }

    let question = "Jaka jest aktualna nazwa mojego projektu i w jakiej jest fazie?";
    let res = client.ask(tonic::Request::new(AskRequest { query: question.to_string() })).await?.into_inner();
    let answer = res.answer.to_lowercase();
    
    println!("Answer: {}", answer);
    assert!(answer.contains("beta") && (answer.contains("ukończony") || answer.contains("wydany")));
    Ok(())
}

#[tokio::test]
async fn test_hallucination_prevention_missing_data() -> Result<()> {
    let mut client = get_client().await?;

    let question = "Kto wygrał wybory na prezydenta Marsa w 2026 roku?";
    let res = client.ask(tonic::Request::new(AskRequest { query: question.to_string() })).await?.into_inner();
    let answer = res.answer.to_lowercase();
    
    println!("Answer: {}", answer);
    let admits_ignorance = answer.contains("nie wiem") || answer.contains("brak") || answer.contains("nie ma") || answer.contains("nie mogę znaleźć");
    assert!(admits_ignorance);
    Ok(())
}
