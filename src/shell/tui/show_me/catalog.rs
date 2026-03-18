use super::db::DocRow;
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CatalogCommand {
    pub subsystem: String,
    pub command: String,
}

pub(crate) type Operations = IndexMap<String, Vec<CatalogCommand>>;

#[derive(Debug, Clone, Default)]
pub(crate) struct CatalogTree {
    pub ops: IndexMap<String, Operations>,
    pub levels: HashMap<String, Vec<String>>,
}

#[must_use]
pub(crate) fn build_catalog(rows: &[DocRow]) -> CatalogTree {
    let mut tree = CatalogTree {
        ops: IndexMap::new(),
        levels: HashMap::with_capacity(rows.len()),
    };

    for row in rows {
        parse_doc_row(row, &mut tree);
    }

    tree
}

fn parse_doc_row(row: &DocRow, tree: &mut CatalogTree) {
    let level = extract_level(&row.fuente);
    let mut current_category = format!("General [{level}]");
    let mut current_operation: Option<String> = None;

    for line in row.texto_completo.lines() {
        let trimmed = line.trim();

        if let Some(category) = trimmed.strip_prefix("# ") {
            let category = category.trim();
            if !category.is_empty() {
                current_category = format!("{category} [{level}]");
                current_operation = None;
                ensure_category_level(tree, &current_category, &level);
                tree.ops.entry(current_category.clone()).or_default();
            }
            continue;
        }

        if let Some(operation) = parse_operation_heading(trimmed) {
            ensure_category_level(tree, &current_category, &level);
            tree.ops
                .entry(current_category.clone())
                .or_default()
                .entry(operation.clone())
                .or_default();
            current_operation = Some(operation);
            continue;
        }

        if trimmed.starts_with("## ") {
            current_operation = None;
            continue;
        }

        let Some(operation_name) = current_operation.as_ref() else {
            continue;
        };

        if let Some(command) = parse_command_line(trimmed) {
            ensure_category_level(tree, &current_category, &level);
            let operations = tree.ops.entry(current_category.clone()).or_default();
            let commands = operations.entry(operation_name.clone()).or_default();
            if !commands.iter().any(|existing| existing == &command) {
                commands.push(command);
            }
        }
    }
}

fn ensure_category_level(tree: &mut CatalogTree, category: &str, level: &str) {
    let levels = tree.levels.entry(category.to_owned()).or_default();
    if !levels.iter().any(|existing| existing == level) {
        levels.push(level.to_owned());
    }
}

fn parse_operation_heading(line: &str) -> Option<String> {
    let heading = line.strip_prefix("## ")?;
    let (label, operation) = heading.split_once(':')?;
    let normalized_label = label.trim().to_lowercase();
    if normalized_label != "intención" && normalized_label != "intencion" {
        return None;
    }

    let operation = operation.trim();
    if operation.is_empty() {
        return None;
    }

    Some(operation.to_owned())
}

fn parse_command_line(line: &str) -> Option<CatalogCommand> {
    let body = line.strip_prefix("- ")?.trim();
    let (raw_subsystem, raw_command) = body.split_once(':')?;
    let subsystem = normalize_subsystem(raw_subsystem.trim())?;
    let command = extract_command(raw_command.trim())?;
    Some(CatalogCommand { subsystem, command })
}

fn normalize_subsystem(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "bash/zsh" | "bash / zsh" => Some("bash/zsh".to_owned()),
        "fish" => Some("fish".to_owned()),
        "powershell" => Some("powershell".to_owned()),
        "cmd" => Some("cmd".to_owned()),
        _ => None,
    }
}

fn extract_command(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }

    if let Some(start) = raw.find('`') {
        let tail = &raw[start + 1..];
        if let Some(end_rel) = tail.find('`') {
            let command = tail[..end_rel].trim();
            if !command.is_empty() {
                return Some(command.to_owned());
            }
        }
    }

    let stripped = raw.trim_matches('`').trim();
    if stripped.is_empty() {
        None
    } else {
        Some(stripped.to_owned())
    }
}

fn extract_level(source: &str) -> String {
    let filename = source.rsplit(['/', '\\']).next().unwrap_or(source);
    let Some(stem) = filename.strip_suffix("-rag.md") else {
        return "General".to_owned();
    };

    let mut parts = stem.split('-');
    let Some(prefix) = parts.next() else {
        return "General".to_owned();
    };
    let Some(level) = parts.next() else {
        return "General".to_owned();
    };

    if parts.next().is_some() || prefix.is_empty() || level.is_empty() {
        return "General".to_owned();
    }

    if !level.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return "General".to_owned();
    }

    capitalize_level(level)
}

fn capitalize_level(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut out = String::new();
    out.push(first.to_ascii_uppercase());
    for ch in chars {
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(fuente: &str, texto_completo: &str) -> DocRow {
        DocRow {
            fuente: fuente.to_owned(),
            texto_completo: texto_completo.to_owned(),
        }
    }

    #[test]
    fn build_catalog_extracts_category_from_h1_markdown() {
        let rows = vec![row(
            "scripting-basico-rag.md",
            "# Filesystem\n\n## Intención: Listar archivos\n- PowerShell: `Get-ChildItem`",
        )];

        let catalog = build_catalog(&rows);

        assert!(catalog.ops.contains_key("Filesystem [Basico]"));
    }

    #[test]
    fn build_catalog_extracts_level_from_filename_pattern() {
        let rows = vec![row(
            "scripting-basico-rag.md",
            "# Filesystem\n\n## Intención: Listar archivos\n- PowerShell: `Get-ChildItem`",
        )];

        let catalog = build_catalog(&rows);
        let levels = catalog
            .levels
            .get("Filesystem [Basico]")
            .cloned()
            .unwrap_or_default();

        assert_eq!(levels, vec!["Basico".to_owned()]);
    }

    #[test]
    fn build_catalog_uses_general_level_for_unrecognized_filename() {
        let rows = vec![row(
            "notes.md",
            "# Filesystem\n\n## Intención: Listar archivos\n- PowerShell: `Get-ChildItem`",
        )];

        let catalog = build_catalog(&rows);
        let levels = catalog
            .levels
            .get("Filesystem [General]")
            .cloned()
            .unwrap_or_default();

        assert_eq!(levels, vec!["General".to_owned()]);
    }

    #[test]
    fn build_catalog_returns_empty_tree_on_no_rows() {
        let catalog = build_catalog(&[]);

        assert!(catalog.ops.is_empty());
        assert!(catalog.levels.is_empty());
    }

    #[test]
    fn build_catalog_extracts_powershell_command_from_bullet_backticks() {
        let rows = vec![row(
            "scripting-avanzado-rag.md",
            "# Procesamiento de Texto Complejo\n\n## Intención: Extraer\n- PowerShell: `Get-Content <ruta_archivo>`",
        )];

        let catalog = build_catalog(&rows);
        let commands = &catalog.ops["Procesamiento de Texto Complejo [Avanzado]"]["Extraer"];
        assert_eq!(
            commands[0],
            CatalogCommand {
                subsystem: "powershell".to_owned(),
                command: "Get-Content <ruta_archivo>".to_owned()
            }
        );
    }

    #[test]
    fn build_catalog_keeps_same_category_name_separated_by_level() {
        let rows = vec![
            row(
                "scripting-basico-rag.md",
                "# Redes\n\n## Intención: Ping\n- PowerShell: `Test-Connection <host>`",
            ),
            row(
                "scripting-medio-rag.md",
                "# Redes\n\n## Intención: DNS\n- PowerShell: `Resolve-DnsName <host>`",
            ),
        ];

        let catalog = build_catalog(&rows);

        assert!(catalog.ops.contains_key("Redes [Basico]"));
        assert!(catalog.ops.contains_key("Redes [Medio]"));
        assert_eq!(catalog.ops.len(), 2);
    }
}
