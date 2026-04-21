mod api;
mod claude_cli;
mod compile;
mod templates;

async fn backend_generate(prompt: &str) -> Result<String> {
    if std::env::var("TEXPLAIN_BACKEND").as_deref() == Ok("claude-cli") {
        claude_cli::generate(prompt).await
    } else {
        api::generate(prompt).await
    }
}

async fn backend_fix(tex: &str, err_log: &str) -> Result<String> {
    if std::env::var("TEXPLAIN_BACKEND").as_deref() == Ok("claude-cli") {
        claude_cli::fix(tex, err_log).await
    } else {
        api::fix(tex, err_log).await
    }
}

use anyhow::{Context, Result, bail};
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

    /// Pula a compilação (só gera o .tex)
    #[arg(long)]
    no_compile: bool,
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

    fs::create_dir_all(&cli.output)
        .with_context(|| format!("erro ao criar {}", cli.output.display()))?;

    let stem = cli.input.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    let tex_path = cli.output.join(format!("{stem}.tex"));

    eprintln!("→ Gerando LaTeX via Claude...");
    let mut tex = backend_generate(&prompt).await?;
    fs::write(&tex_path, &tex)
        .with_context(|| format!("erro ao gravar {}", tex_path.display()))?;
    eprintln!("✓ .tex inicial em {}", tex_path.display());

    if cli.no_compile {
        return Ok(());
    }

    for attempt in 1..=(cli.max_retries + 1) {
        eprintln!("→ Compilando (tentativa {attempt}/{})...", cli.max_retries + 1);
        match compile::compile(&tex_path, &cli.output)? {
            Ok(pdf) => {
                eprintln!("✓ PDF gerado em {}", pdf.display());
                return Ok(());
            }
            Err(err_log) => {
                if attempt > cli.max_retries {
                    eprintln!("✗ Falhou após {} tentativas.", cli.max_retries + 1);
                    eprintln!("--- último log ---\n{err_log}");
                    bail!("compilação não convergiu");
                }
                eprintln!("✗ Erro de compilação, pedindo correção à IA...");
                tex = backend_fix(&tex, &err_log).await?;
                fs::write(&tex_path, &tex)
                    .with_context(|| format!("erro ao gravar {}", tex_path.display()))?;
            }
        }
    }

    Ok(())
}
