use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Resultado da compilação: Ok(pdf_path) ou Err(log_de_erro_tail).
pub fn compile(tex_path: &Path, out_dir: &Path) -> Result<Result<PathBuf, String>> {
    let output = Command::new("latexmk")
        .arg("-pdf")
        .arg("-interaction=nonstopmode")
        .arg("-halt-on-error")
        .arg(format!("-output-directory={}", out_dir.display()))
        .arg(tex_path)
        .output()
        .context("falha ao executar latexmk (está instalado e no PATH?)")?;

    if output.status.success() {
        let stem = tex_path
            .file_stem()
            .and_then(|s| s.to_str())
            .context("nome de arquivo inválido")?;
        let pdf = out_dir.join(format!("{stem}.pdf"));
        return Ok(Ok(pdf));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let tail = tail_lines(&stdout, 40);
    let err_log = format!("STDOUT (últimas 40 linhas):\n{tail}\n\nSTDERR:\n{stderr}");

    Ok(Err(err_log))
}

fn tail_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}
