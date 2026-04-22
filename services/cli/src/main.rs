use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub mod tscherepacha {
    tonic::include_proto!("tscherepacha");
}

use tscherepacha::orchestrator_client::OrchestratorClient;
use tscherepacha::{IngestRequest, AskRequest, FileContent};
use tscherepacha::ingest_request::Source;

#[derive(Parser)]
#[command(name = "tsch")]
#[command(about = "Tscherepacha AI Second Brain CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Orchestrator gRPC URL
    #[arg(short, long, default_value = "http://localhost:50052")]
    url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest knowledge into the second brain
    Ingest {
        /// Direct text to ingest
        #[arg(short, long)]
        text: Option<String>,

        /// Path to a file (PDF, PNG, JPG) to ingest
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// Ask a question based on your stored knowledge
    Ask {
        /// The question you want to ask
        query: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut client = OrchestratorClient::connect(cli.url.clone())
        .await
        .with_context(|| format!("Failed to connect to Orchestrator at {}", cli.url))?;

    match cli.command {
        Commands::Ingest { text, file } => {
            let source = if let Some(t) = text {
                Source::Text(t)
            } else if let Some(f) = file {
                let content = fs::read(&f)
                    .with_context(|| format!("Failed to read file: {:?}", f))?;
                
                let extension = f.extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();
                
                let file_name = f.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                Source::File(FileContent {
                    content,
                    file_type: extension,
                    file_name,
                })
            } else {
                return Err(anyhow::anyhow!("Either --text or --file must be provided"));
            };

            let request = tonic::Request::new(IngestRequest { source: Some(source) });
            let response = client.ingest(request).await?.into_inner();

            if response.success {
                println!("Success: {}", response.message);
            } else {
                println!("Error: Ingestion failed: {}", response.message);
            }
        }
        Commands::Ask { query } => {
            let request = tonic::Request::new(AskRequest { query });
            let response = client.ask(request).await?.into_inner();

            println!("\nTscherepacha Answer:");
            println!("-------------------");
            println!("{}", response.answer);
        }
    }

    Ok(())
}
