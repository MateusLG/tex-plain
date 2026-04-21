use crate::Template;

const ARTIGO: &str = include_str!("../templates/artigo.txt");
const ANOTACAO: &str = include_str!("../templates/anotacao.txt");
const ESTUDO_DIRIGIDO: &str = include_str!("../templates/estudo-dirigido.txt");

pub fn build_prompt(template: &Template, summary: &str) -> String {
    let raw = match template {
        Template::Artigo => ARTIGO,
        Template::Anotacao => ANOTACAO,
        Template::EstudoDirigido => ESTUDO_DIRIGIDO,
    };

    raw.replace("{{SUMMARY}}", summary)
}
