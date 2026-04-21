mod templates;

use clap::{Parser, ValueEnum};
use std::fs;
use std::path::PathBuf;
use std::process;

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

fn main() {
    let cli = Cli::parse();

    let summary = match fs::read_to_string(&cli.input) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Erro ao ler {}: {}", cli.input.display(), e);
            process::exit(1);
        }
    };

    let prompt = templates::build_prompt(&cli.template, &summary);

    println!("--- PROMPT GERADO ---");
    println!("{}", prompt);
}
