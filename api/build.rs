use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = PathBuf::from(&manifest_dir).join("assets").join("icons");
    fs::create_dir_all(&out_dir).unwrap();

    let icons_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .join("web")
        .join("assets")
        .join("icons");

    generate_sprite(&icons_dir.join("ui"), &out_dir.join("sprite-ui.svg"), "ui");
    generate_sprite(
        &icons_dir.join("narrative"),
        &out_dir.join("sprite-narrative.svg"),
        "narrative",
    );

    println!("cargo:rerun-if-changed=../web/assets/icons/ui/");
    println!("cargo:rerun-if-changed=../web/assets/icons/narrative/");
}

fn generate_sprite(input_dir: &Path, output_path: &Path, category: &str) {
    let mut symbols = Vec::new();

    if input_dir.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(input_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "svg"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let content = fs::read_to_string(&path).unwrap();
            let file_name = path
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .replace('-', "_");
            let id = format!("icon_{}_{}", category, file_name);

            // Extract inner content from <svg>...</svg>
            let inner = extract_svg_inner(&content);
            symbols.push(format!(
                r#"<symbol id="{}" viewBox="0 0 24 24" fill="currentColor">{}</symbol>"#,
                id, inner
            ));
        }
    }

    let sprite = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none">
{}
</svg>"#,
        symbols.join("\n")
    );

    fs::write(output_path, &sprite).unwrap();
    println!(
        "cargo:warning=Generated {} sprite with {} icons",
        category,
        symbols.len()
    );
}

fn extract_svg_inner(svg: &str) -> String {
    // Find content between first > and last </svg>
    if let Some(start) = svg.find('>') {
        let rest = &svg[start + 1..];
        if let Some(end) = rest.rfind("</svg>") {
            return rest[..end].trim().to_string();
        }
    }
    svg.to_string()
}
