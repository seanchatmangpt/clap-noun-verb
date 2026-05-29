//! Command routing logic for noun-verb CLI

use crate::error::{NounVerbError, Result};
use crate::noun::NounCommand;
use crate::verb::{VerbArgs, VerbContext, TypeMap};
use crate::middleware::{MiddlewarePipeline, MiddlewareRequest, MiddlewareResponse};
use clap::{ArgMatches, Command};
use std::collections::HashMap;

/// Router for dispatching noun-verb commands
pub struct CommandRouter {
    nouns: HashMap<String, Box<dyn NounCommand>>,
    extensions: TypeMap,
    pipeline: Option<MiddlewarePipeline>,
}

impl CommandRouter {
    /// Create a new command router
    pub fn new() -> Self {
        Self { 
            nouns: HashMap::new(),
            extensions: TypeMap::new(),
            pipeline: None,
        }
    }

    /// Add a typed extension to the global context
    pub fn with_extension<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.extensions.insert(val);
        self
    }

    /// Register a noun command
    pub fn register_noun(&mut self, noun: Box<dyn NounCommand>) {
        self.nouns.insert(noun.name().to_string(), noun);
    }

    /// Set the middleware pipeline
    pub fn with_pipeline(mut self, pipeline: MiddlewarePipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Route a command based on clap matches
    pub fn route(&self, matches: &ArgMatches) -> Result<()> {
        // Get the top-level subcommand (noun)
        let (noun_name, noun_matches) = matches
            .subcommand()
            .ok_or_else(|| NounVerbError::invalid_structure("No subcommand found"))?;

        // Find the noun command
        let noun =
            self.nouns.get(noun_name).ok_or_else(|| {
                let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
                NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
            })?;

        // Route the command recursively with root matches for global args
        self.route_recursive(noun.as_ref(), noun_name, noun_matches, matches)
    }

    /// Recursively route commands through nested noun-verb structure
    #[allow(clippy::only_used_in_recursion)]
    fn route_recursive(
        &self,
        noun: &dyn NounCommand,
        noun_name: &str,
        matches: &ArgMatches,
        root_matches: &ArgMatches,
    ) -> Result<()> {
        // Check if there's a subcommand (either verb or sub-noun)
        if let Some((sub_name, sub_matches)) = matches.subcommand() {
            // First check if it's a verb
            if let Some(verb) = noun.verbs().iter().find(|v| v.name() == sub_name) {
                // Execute the verb with root matches for global args access
                let mut context = VerbContext::new(sub_name).with_noun(noun_name);
                context.extensions = self.extensions.clone();
                let args = VerbArgs::new(sub_matches.clone())
                    .with_parent(root_matches.clone())
                    .with_context(context);

                if let Some(pipeline) = &self.pipeline {
                    let mut req = MiddlewareRequest::new(sub_name);
                    for arg in sub_matches.ids() {
                        if let Some(vals) = sub_matches.get_many::<String>(arg.as_str()) {
                            for val in vals {
                                req = req.with_arg(val);
                            }
                        }
                    }
                    if let Err(e) = pipeline.execute_before(&req) {
                        return Err(NounVerbError::execution_error(format!("Middleware rejected request: {}", e)));
                    }
                }

                let result = verb.run(&args);

                if let Some(pipeline) = &self.pipeline {
                    match &result {
                        Ok(_) => {
                            let _ = pipeline.execute_after(&MiddlewareResponse::success("Success"));
                        }
                        Err(e) => {
                            let _ = pipeline.execute_after(&MiddlewareResponse::failure(e.to_string()));
                            if let Ok(Some(_)) = pipeline.handle_error(e) {
                                // If middleware recovered, we could theoretically return Ok(())
                                // but for now we'll just let it handle the error state.
                            }
                        }
                    }
                }

                result
            } else if let Some(sub_noun) = noun.sub_nouns().iter().find(|n| n.name() == sub_name) {
                // Recursively route to sub-noun, passing root matches for global args
                self.route_recursive(sub_noun.as_ref(), sub_name, sub_matches, root_matches)
            } else {
                // Neither verb nor sub-noun found
                let mut candidates: Vec<&str> = noun.verbs().iter().map(|v| v.name()).collect();
                candidates.extend(noun.sub_nouns().iter().map(|n| n.name()));
                Err(NounVerbError::verb_not_found_with_candidates(noun_name, sub_name, &candidates))
            }
        } else {
            // No subcommand, try direct noun execution
            let mut context = VerbContext::new("").with_noun(noun_name);
            context.extensions = self.extensions.clone();
            let args = VerbArgs::new(matches.clone()).with_context(context);

            noun.handle_direct(&args)
        }
    }

    /// Build the complete clap command structure
    pub fn build_command(&self, app_name: &'static str, about: &'static str) -> Command {
        let mut cmd = Command::new(app_name).about(about);

        for noun in self.nouns.values() {
            cmd = cmd.subcommand(noun.build_command());
        }

        cmd
    }

    /// Get all registered noun names
    pub fn noun_names(&self) -> Vec<&str> {
        self.nouns.keys().map(|s| s.as_str()).collect()
    }

    /// Get verbs for a specific noun
    pub fn get_verbs(&self, noun_name: &str) -> Result<Vec<String>> {
        let noun =
            self.nouns.get(noun_name).ok_or_else(|| {
                let candidates: Vec<&str> = self.nouns.keys().map(|s| s.as_str()).collect();
                NounVerbError::command_not_found_with_candidates(noun_name, &candidates)
            })?;

        Ok(noun.verbs().iter().map(|v| v.name().to_string()).collect())
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new()
    }
}
