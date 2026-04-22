mod storage;
mod llm;
mod ocr;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Tscherepacha Orchestrator starting up...");

    // TODO: Implement CLI with 'clap'
    // - tsch ingest <file>
    // - tsch ask <question>
    
    Ok(())
}
