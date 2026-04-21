use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-sonnet-4-6";
const MAX_TOKENS: u32 = 8192;

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

pub async fn generate(prompt: &str) -> Result<String> {
    call(prompt).await
}

pub async fn fix(tex: &str, error_log: &str) -> Result<String> {
    let prompt = format!(
        "O seguinte arquivo LaTeX falhou ao compilar. Corrija os erros e retorne APENAS o código LaTeX corrigido, sem explicações, sem markdown fences, sem texto fora do .tex.\n\n=== .tex atual ===\n{tex}\n\n=== log de erro ===\n{error_log}\n"
    );
    call(&prompt).await
}

async fn call(prompt: &str) -> Result<String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .context("ANTHROPIC_API_KEY não está definida no ambiente")?;

    let body = Request {
        model: DEFAULT_MODEL,
        max_tokens: MAX_TOKENS,
        messages: vec![Message { role: "user", content: prompt }],
    };

    let client = reqwest::Client::new();
    let res = client
        .post(API_URL)
        .header("x-api-key", api_key)
        .header("anthropic-version", API_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .context("falha ao enviar requisição para a API")?;

    let status = res.status();
    if !status.is_success() {
        let err_body = res.text().await.unwrap_or_default();
        bail!("API retornou {}: {}", status, err_body);
    }

    let parsed: Response = res.json().await.context("falha ao parsear resposta JSON")?;

    parsed
        .content
        .into_iter()
        .find(|b| b.block_type == "text")
        .and_then(|b| b.text)
        .context("resposta da API não contém bloco de texto")
}
