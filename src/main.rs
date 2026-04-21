mod api;
mod templates;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "tex-plain", version, about = "Gera documentos LaTeX a partir de resumos usando IA")]
struct Cli {
    /// Arquivo de entrada com o resumo (markdown ou texto)
    input: PathBuf,

    /// Template a usar
    #[arg(short, long, value_enum, default_value_t = Template::Anotacao)]
    template: Template,

    /// Diretório de saída (default: diretório atual)
    #[arg(short, long, default_value = ".")]
    output: PathBuf,

    /// Máximo de tentativas de auto-fix em caso de erro de compilação
    #[arg(long, default_value_t = 3)]
    max_retries: u8,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Template {
    Artigo,
    Anotacao,
    EstudoDirigido,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    let summary = fs::read_to_string(&cli.input)
        .with_context(|| format!("erro ao ler {}", cli.input.display()))?;

    let prompt = templates::build_prompt(&cli.template, &summary);

    eprintln!("→ Gerando LaTeX via Claude...");
    let tex = api::generate(&prompt).await?;

    fs::create_dir_all(&cli.output)
        .with_context(|| format!("erro ao criar {}", cli.output.display()))?;

    let stem = cli.input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let tex_path = cli.output.join(format!("{stem}.tex"));

    fs::write(&tex_path, &tex)
        .with_context(|| format!("erro ao gravar {}", tex_path.display()))?;

    eprintln!("✓ Escrito em {}", tex_path.display());
    Ok(())
}
