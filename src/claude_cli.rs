use anyhow::{Context, Result, bail};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

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
    let mut child = Command::new("claude")
        .arg("-p")
        .arg("--output-format=text")
        .env_remove("ANTHROPIC_API_KEY")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("falha ao iniciar `claude` CLI (instalado e no PATH?)")?;

    let mut stdin = child
        .stdin
        .take()
        .context("stdin do claude indisponível")?;
    stdin
        .write_all(prompt.as_bytes())
        .await
        .context("falha ao escrever prompt no stdin do claude")?;
    drop(stdin);

    let output = child
        .wait_with_output()
        .await
        .context("falha ao aguardar saída do claude")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        bail!(
            "claude CLI falhou (status={:?})\nstderr: {stderr}\nstdout: {stdout}",
            output.status.code()
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}
