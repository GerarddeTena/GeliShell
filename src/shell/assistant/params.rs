#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssistantParameter {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub prompt_prefix: &'static str,
}

pub const PREDEFINED_PARAMETERS: &[AssistantParameter] = &[
    AssistantParameter {
        id: "output-my-code",
        label: "Output my code",
        description: "Explain generated command output and next safe step",
        prompt_prefix: "Explain current command output and provide a safe follow-up command. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
    AssistantParameter {
        id: "copy-directories",
        label: "Copy directories",
        description: "Generate recursive copy commands with safe flags",
        prompt_prefix: "Create a safe recursive directory copy command for this shell. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
    AssistantParameter {
        id: "search-in-files",
        label: "Search in files",
        description: "Build fast file-content search command templates",
        prompt_prefix: "Generate an efficient file search command with rg-first strategy. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
    AssistantParameter {
        id: "compress-extract",
        label: "Compress/Extract",
        description: "Suggest archive create/extract commands per subsystem",
        prompt_prefix: "Generate archive compression/extraction commands for this subsystem. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
    AssistantParameter {
        id: "network-request",
        label: "Network request",
        description: "Prepare curl/Invoke-WebRequest command templates",
        prompt_prefix: "Create a safe network request command template with clear placeholders. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
    AssistantParameter {
        id: "process-management",
        label: "Process management",
        description: "List/stop processes with explicit safety checks",
        prompt_prefix: "Generate process listing and termination commands with safety notes. Base your answer only on canonical commands from the retrieved guardrail context.",
    },
];

pub fn filter_parameters(filter: &str) -> Vec<AssistantParameter> {
    let normalized = filter.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return PREDEFINED_PARAMETERS.to_vec();
    }

    PREDEFINED_PARAMETERS
        .iter()
        .copied()
        .filter(|item| {
            item.label.to_ascii_lowercase().contains(&normalized)
                || item.description.to_ascii_lowercase().contains(&normalized)
                || item.id.to_ascii_lowercase().contains(&normalized)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_returns_all_on_empty_query() {
        let filtered = filter_parameters("");
        assert_eq!(filtered.len(), PREDEFINED_PARAMETERS.len());
    }

    #[test]
    fn filter_matches_partial_label() {
        let filtered = filter_parameters("copy");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "copy-directories");
    }

    #[test]
    fn filter_matches_description() {
        let filtered = filter_parameters("archive");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "compress-extract");
    }
}
