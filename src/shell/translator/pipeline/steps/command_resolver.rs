use crate::shell::reporter::Reporter;
use crate::parser::lexer::Lexer;
use crate::parser::token::Token;
use crate::shell::translator::commands_map::CommandMap;
use crate::shell::translator::commands_map::CommandDef;
use crate::shell::translator::pipeline::context::TranslationContext;
use crate::shell::translator::pipeline::step::{PipelineError, StepResult, TranslationStep};
use crate::t;

pub struct CommandResolver;

impl CommandResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommandResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
struct ReverseLookupMatch {
    def: CommandDef,
    consumed_args: usize,
}

fn tokens_for_exact(exact: &str) -> Vec<String> {
    Lexer::new(exact)
        .tokenize()
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|token| token.as_str().map(str::to_owned))
        .collect()
}

fn fragment_tokens(fragment: &crate::shell::translator::pipeline::context::CommandFragment) -> Vec<String> {
    std::iter::once(fragment.command.clone())
        .chain(
            fragment
                .args
                .iter()
                .filter_map(|token| token.as_str().map(str::to_owned)),
        )
        .collect()
}

fn reverse_lookup_match(map: &CommandMap, fragment: &crate::shell::translator::pipeline::context::CommandFragment) -> Option<ReverseLookupMatch> {
    let current_tokens = fragment_tokens(fragment);
    let mut best: Option<(usize, CommandDef)> = None;

    for def in map.iter() {
        for (_, entry_opt) in def.translate.iter_named() {
            let Some(entry) = entry_opt else { continue };
            let exact_tokens = tokens_for_exact(&entry.exact);
            if exact_tokens.is_empty() || exact_tokens.len() > current_tokens.len() {
                continue;
            }
            if current_tokens[..exact_tokens.len()] != exact_tokens[..] {
                continue;
            }

            match &best {
                None => best = Some((exact_tokens.len(), def.clone())),
                Some((best_len, best_def)) if exact_tokens.len() > *best_len => {
                    best = Some((exact_tokens.len(), def.clone()));
                }
                Some((best_len, best_def)) if exact_tokens.len() == *best_len && best_def.name != def.name => {
                    best = None;
                }
                _ => {}
            }
        }
    }

    best.map(|(len, def)| ReverseLookupMatch {
        def,
        consumed_args: len.saturating_sub(1),
    })
}

fn normalize_reverse_lookup_args(def: &CommandDef, args: &[Token], consumed_args: usize) -> Vec<Token> {
    args.iter()
        .skip(consumed_args)
        .map(|token| {
            let Some(text) = token.as_str() else {
                return token.clone();
            };

            for flag in &def.flags {
                if flag.bash.as_deref() == Some(text)
                    || flag.zsh.as_deref() == Some(text)
                    || flag.fish.as_deref() == Some(text)
                    || flag.powershell.as_deref() == Some(text)
                    || flag.cmd.as_deref() == Some(text)
                {
                    return Token::Word(flag.canonical.clone());
                }
            }

            token.clone()
        })
        .collect()
}

impl TranslationStep for CommandResolver {
    fn name(&self) -> &'static str {
        "CommandResolver"
    }

    fn process(
        &self,
        ctx: &mut TranslationContext,
        reporter: &dyn Reporter,
    ) -> Result<StepResult, PipelineError> {
        for fragment in ctx.fragments.iter_mut() {
            let native_signature = fragment.to_native_string(ctx.subsystem);
            let reverse_match = reverse_lookup_match(ctx.map, fragment);

            match ctx.map.get(&fragment.command) {
                Some(def) => {
                    reporter.info(&t!(
                        "pipeline.canonical_match",
                        step = self.name(),
                        command = fragment.command
                    ));
                    fragment.command_def = Some(def.clone());
                }
                None => {
                    if let Some(matched) = reverse_match {
                        reporter.info(&t!(
                            "pipeline.reverse_lookup",
                            step = self.name(),
                            command = native_signature,
                            canonical = matched.def.name
                        ));
                        fragment.command = matched.def.name.clone();
                        fragment.args = normalize_reverse_lookup_args(
                            &matched.def,
                            &fragment.args,
                            matched.consumed_args,
                        );
                        fragment.command_def = Some(matched.def);
                    } else {
                        // Pass-through — comando nativo sin equivalente canónico
                        reporter.info(&t!(
                            "pipeline.passthrough",
                            step = self.name(),
                            command = native_signature
                        ));
                    }
                }
            }
        }

        ctx.snapshot(self.name());
        Ok(StepResult::Continue)
    }
}
