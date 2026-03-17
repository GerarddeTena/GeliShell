use super::params::AssistantParameter;
use super::rag::RagChunk;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantSuggestion {
    pub title: String,
    pub body: String,
    pub sources: Vec<String>,
}

pub fn build_user_prompt(parameter: AssistantParameter, filter: &str) -> String {
    let trimmed_filter = filter.trim();
    if trimmed_filter.is_empty() {
        parameter.prompt_prefix.to_owned()
    } else {
        format!(
            "{} Filter hint: {}",
            parameter.prompt_prefix, trimmed_filter
        )
    }
}

pub fn build_system_prompt(parameter: AssistantParameter, rag_context: &[RagChunk]) -> String {
    let mut out = String::new();
    out.push_str("You are the local GeliShell assistant.\n");
    out.push_str("Return actionable shell guidance with explicit safety constraints.\n");
    out.push_str("Basa tu respuesta estrictamente en los comandos canónicos proporcionados en el contexto. No sugieras binarios destructivos fuera del Guardrail.\n");
    out.push_str("[CONTEXTO DE SEGURIDAD Y COMANDOS PERMITIDOS]\n");
    out.push_str(&format!(
        "Acción solicitada: {} ({})\n",
        parameter.label, parameter.id
    ));

    if rag_context.is_empty() {
        out.push_str(
            "No se recuperaron chunks vectoriales para esta consulta. Responde de forma conservadora y no inventes comandos fuera del diccionario canónico.\n",
        );
    } else {
        for chunk in rag_context {
            out.push_str(&format!(
                "- Fuente: {} | Distancia coseno: {:.4}\n{}\n",
                chunk.path, chunk.distance, chunk.text
            ));
        }
    }

    out
}

pub fn build_suggestion(
    parameter: AssistantParameter,
    generated: String,
    rag_context: &[RagChunk],
) -> AssistantSuggestion {
    let sources = rag_context
        .iter()
        .map(|chunk| chunk.path.clone())
        .collect::<Vec<_>>();

    AssistantSuggestion {
        title: format!("GeliShell Assistant — {}", parameter.label),
        body: generated,
        sources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_prompt_includes_filter_when_present() {
        let parameter = AssistantParameter {
            id: "search",
            label: "Search in files",
            description: "desc",
            prompt_prefix: "Base prompt.",
        };
        let prompt = build_user_prompt(parameter, "rg config");
        assert!(prompt.contains("Base prompt."));
        assert!(prompt.contains("rg config"));
    }

    #[test]
    fn system_prompt_contains_guardrail_header() {
        let parameter = AssistantParameter {
            id: "copy",
            label: "Copy directories",
            description: "desc",
            prompt_prefix: "Base prompt.",
        };
        let context = vec![RagChunk {
            path: "docs/kb/guardrail.md".to_owned(),
            text: "Only canonical commands are allowed.".to_owned(),
            distance: 0.1234,
        }];

        let prompt = build_system_prompt(parameter, &context);
        assert!(prompt.contains("[CONTEXTO DE SEGURIDAD Y COMANDOS PERMITIDOS]"));
        assert!(prompt.contains("Basa tu respuesta estrictamente"));
    }

    #[test]
    fn suggestion_collects_sources() {
        let parameter = AssistantParameter {
            id: "copy",
            label: "Copy directories",
            description: "desc",
            prompt_prefix: "Prompt",
        };
        let context = vec![RagChunk {
            path: "docs/guide.md".to_owned(),
            text: "snippet".to_owned(),
            distance: 0.2,
        }];

        let suggestion = build_suggestion(parameter, "answer".to_owned(), &context);
        assert_eq!(suggestion.sources, vec!["docs/guide.md".to_owned()]);
    }
}
