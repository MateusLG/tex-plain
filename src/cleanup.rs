use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

const ARTIFACT_EXTS: &[&str] = &[
    "aux",
    "log",
    "fdb_latexmk",
    "fls",
    "out",
    "toc",
    "bbl",
    "blg",
    "synctex.gz",
    "nav",
    "snm",
    "vrb",
];

/// Move .pdf e .tex pra subpastas, apaga artefatos temporários.
/// Retorna (novo_pdf, novo_tex).
pub fn organize(out_dir: &Path, stem: &str) -> Result<()> {
    let pdf_dir = out_dir.join("pdf");
    let tex_dir = out_dir.join("tex");
    fs::create_dir_all(&pdf_dir).with_context(|| format!("criar {}", pdf_dir.display()))?;
    fs::create_dir_all(&tex_dir).with_context(|| format!("criar {}", tex_dir.display()))?;

    let pdf_src = out_dir.join(format!("{stem}.pdf"));
    let pdf_dst = pdf_dir.join(format!("{stem}.pdf"));
    if pdf_src.exists() {
        fs::rename(&pdf_src, &pdf_dst)
            .with_context(|| format!("mover {} → {}", pdf_src.display(), pdf_dst.display()))?;
    }

    let tex_src = out_dir.join(format!("{stem}.tex"));
    let tex_dst = tex_dir.join(format!("{stem}.tex"));
    if tex_src.exists() {
        fs::rename(&tex_src, &tex_dst)
            .with_context(|| format!("mover {} → {}", tex_src.display(), tex_dst.display()))?;
    }

    for ext in ARTIFACT_EXTS {
        let path = out_dir.join(format!("{stem}.{ext}"));
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }

    Ok(())
}
