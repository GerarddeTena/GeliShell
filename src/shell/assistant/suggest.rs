use super::params::AssistantParameter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantSuggestion {
    pub body: String,
}

pub fn build_retrieval_query(parameter: AssistantParameter, filter: &str) -> String {
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

pub fn build_user_action(parameter: AssistantParameter, _filter: &str) -> String {
    parameter.label.to_owned()
}

pub fn build_chatml_prompt(user_action: &str, rag_context: &str) -> String {
    format!(
        "<|im_start|>system\nEres GeliShell Assistant, una IA integrada en una terminal de consola. Tu tarea es responder a la acción del usuario de forma ultra-concisa y directa.\nREGLA DE ORO: NO escupas ni repitas el texto crudo de la documentación. Úsalo solo como conocimiento interno para sintetizar el comando o la solución exacta que pide el usuario.\nResponde solo con comandos aplicables o con una explicación de 1 o 2 líneas como máximo.\n[CONTEXTO RECUPERADO DE RAG]\n{rag_context}\n<|im_end|>\n<|im_start|>user\n{user_action}\n<|im_end|>\n<|im_start|>assistant"
    )
}

pub fn build_suggestion(generated: String) -> AssistantSuggestion {
    AssistantSuggestion { body: generated }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieval_query_includes_filter_when_present() {
        let parameter = AssistantParameter {
            id: "search",
            label: "Search in files",
            description: "desc",
            prompt_prefix: "Base prompt.",
        };
        let prompt = build_retrieval_query(parameter, "rg config");
        assert!(prompt.contains("Base prompt."));
        assert!(prompt.contains("rg config"));
    }

    #[test]
    fn user_action_prefers_selected_label() {
        let parameter = AssistantParameter {
            id: "copy",
            label: "Copy directories",
            description: "desc",
            prompt_prefix: "Base prompt.",
        };

        assert_eq!(build_user_action(parameter, ""), "Copy directories");
    }

    #[test]
    fn chatml_prompt_contains_strict_sections() {
        let prompt = build_chatml_prompt("Search in files", "doc chunk 1");
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("[CONTEXTO RECUPERADO DE RAG]"));
        assert!(prompt.contains("doc chunk 1"));
        assert!(prompt.contains("<|im_start|>user\nSearch in files\n<|im_end|>"));
        assert!(prompt.ends_with("<|im_start|>assistant"));
    }

    #[test]
    fn suggestion_wraps_generated_body() {
        let suggestion = build_suggestion("answer".to_owned());
        assert_eq!(suggestion.body, "answer");
    }
}
