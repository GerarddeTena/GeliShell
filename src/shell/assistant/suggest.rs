use crate::t;
use super::params::AssistantParameter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssistantSuggestion {
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HowToSuggestion {
    pub explanation: String,
    pub command: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SuggestionError {
    #[error("gerisabet --how-to requires a non-empty query")]
    EmptyHowToQuery,

    #[error("how-to output missing EXPLANATION line")]
    MissingExplanation,

    #[error("how-to output missing COMMAND line")]
    MissingCommand,

    #[error("how-to output must contain exactly two non-empty lines")]
    InvalidHowToFormat,
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
    t!(
        "assistant.prompt.show_me",
        rag_context = rag_context,
        user_action = user_action
    )
}

pub fn build_suggestion(generated: String) -> AssistantSuggestion {
    AssistantSuggestion { body: generated }
}

pub fn build_how_to_retrieval_query(
    query: &str,
    subsystem: &str,
) -> Result<String, SuggestionError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(SuggestionError::EmptyHowToQuery);
    }

    Ok(format!(
        "Subsystem target: {subsystem}. User goal: {trimmed}"
    ))
}

pub fn build_how_to_chatml_prompt(subsystem: &str, rag_context: &str, query: &str) -> String {
    t!(
        "assistant.prompt.how_to",
        subsystem = subsystem,
        rag_context = rag_context,
        query = query
    )
}

pub fn parse_how_to_response(raw: &str) -> Result<HowToSuggestion, SuggestionError> {
    let lines: Vec<&str> = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() != 2 {
        return Err(SuggestionError::InvalidHowToFormat);
    }

    let explanation = lines[0]
        .strip_prefix("EXPLANATION:")
        .ok_or(SuggestionError::MissingExplanation)?
        .trim()
        .to_owned();

    let command = lines[1]
        .strip_prefix("COMMAND:")
        .ok_or(SuggestionError::MissingCommand)?
        .trim()
        .to_owned();

    if explanation.is_empty() {
        return Err(SuggestionError::MissingExplanation);
    }

    if command.is_empty() {
        return Err(SuggestionError::MissingCommand);
    }

    Ok(HowToSuggestion {
        explanation,
        command,
    })
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
        assert!(prompt.contains("[CONTEXT]"));
        assert!(prompt.contains("[END CONTEXT]"));
        assert!(prompt.contains("doc chunk 1"));
        assert!(prompt.contains("<|im_start|>user\nSearch in files\n<|im_end|>"));
        assert!(prompt.ends_with("<|im_start|>assistant\n"));
    }

    #[test]
    fn suggestion_wraps_generated_body() {
        let suggestion = build_suggestion("answer".to_owned());
        assert_eq!(suggestion.body, "answer");
    }

    #[test]
    fn build_how_to_retrieval_query_rejects_empty_query() {
        let result = build_how_to_retrieval_query("   ", "powershell");
        assert_eq!(result, Err(SuggestionError::EmptyHowToQuery));
    }

    #[test]
    fn build_how_to_prompt_contains_strict_format_contract() {
        let prompt = build_how_to_chatml_prompt("powershell", "ctx", "listar archivos");
        assert!(prompt.contains("subsystem: powershell"));
        assert!(prompt.contains("[CONTEXT]"));
        assert!(prompt.contains("[END CONTEXT]"));
        assert!(prompt.contains("EXPLANATION: [Your one-line explanation]"));
        assert!(prompt.contains("COMMAND: [The command extracted from context]"));
    }

    #[test]
    fn parse_how_to_response_accepts_two_line_contract() {
        let parsed = parse_how_to_response(
            "EXPLANATION: Lista archivos del directorio actual\nCOMMAND: Get-ChildItem -Force <ruta_directorio>",
        )
        .unwrap();

        assert_eq!(
            parsed,
            HowToSuggestion {
                explanation: "Lista archivos del directorio actual".to_owned(),
                command: "Get-ChildItem -Force <ruta_directorio>".to_owned(),
            }
        );
    }

    #[test]
    fn parse_how_to_response_rejects_extra_lines() {
        let parsed = parse_how_to_response("EXPLANATION: ok\nCOMMAND: ls -la <ruta>\nNOTA: extra");
        assert_eq!(parsed, Err(SuggestionError::InvalidHowToFormat));
    }
}
