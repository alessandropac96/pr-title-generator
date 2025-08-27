//! Machine learning model integration for PR title generation

use crate::{context::CleanContext, Error, GeneratorConfig, Result};
use regex::Regex;
use std::collections::HashMap;

/// ML-based PR title generator
pub struct TitleGenerator {
    config: GeneratorConfig,
    // For now, we'll use pattern-based generation
    // TODO: Replace with actual ML model integration using candle-rs
    patterns: PatternMatcher,
}

impl TitleGenerator {
    pub fn new(config: GeneratorConfig) -> Result<Self> {
        // Validate configuration
        if config.temperature < 0.1 || config.temperature > 1.0 {
            return Err(Error::InvalidTemperature {
                temp: config.temperature,
            });
        }
        
        if config.max_length == 0 {
            return Err(Error::InvalidMaxLength {
                length: config.max_length,
            });
        }
        
        // Validate model name
        let supported_models = ["tiny-llama", "phi-2", "gemma-2b", "llama-2-7b"];
        if !supported_models.contains(&config.model_name.as_str()) {
            return Err(Error::UnsupportedModel {
                name: config.model_name.clone(),
            });
        }
        
        let patterns = PatternMatcher::new()?;
        
        if config.verbose {
            println!("Initialized title generator with model: {}", config.model_name);
        }
        
        Ok(Self { config, patterns })
    }
    
    /// Generate a PR title from the given context
    pub async fn generate_title(&self, context: &CleanContext) -> Result<String> {
        if self.config.verbose {
            println!("Generating title with context: {:#?}", context);
        }
        
        // For now, use pattern-based generation
        // TODO: Replace with actual ML model inference
        let title = self.patterns.generate_title(context, &self.config)?;
        
        let processed_title = self.post_process_title(title, context)?;
        
        if self.config.verbose {
            println!("Generated title: {}", processed_title);
        }
        
        Ok(processed_title)
    }
    
    /// Post-process the generated title
    fn post_process_title(&self, mut title: String, context: &CleanContext) -> Result<String> {
        // Ensure title is not too long
        if title.len() > self.config.max_length {
            title = format!("{}...", &title[..self.config.max_length.saturating_sub(3)]);
        }
        
        // Add ticket number if not present and we have one
        if let Some(ticket) = &context.ticket {
            if !title.contains(ticket) && !self.is_generic_title(&title) {
                title = format!("{}: {}", ticket, title);
            }
        }
        
        // Ensure proper capitalization
        title = self.capitalize_title(&title);
        
        // Final length check after adding ticket
        if title.len() > 72 {
            title = format!("{}...", &title[..69]);
        }
        
        Ok(title)
    }
    
    /// Check if a title is too generic
    fn is_generic_title(&self, title: &str) -> bool {
        let generic_terms = ["update", "change", "modify", "fix", "improve"];
        let words: Vec<&str> = title.split_whitespace().collect();
        
        words.len() <= 2 || words.iter().all(|word| {
            generic_terms.contains(&word.to_lowercase().as_str()) || word.len() <= 3
        })
    }
    
    /// Capitalize the first letter of each sentence
    fn capitalize_title(&self, title: &str) -> String {
        let mut chars: Vec<char> = title.chars().collect();
        if let Some(first_char) = chars.first_mut() {
            *first_char = first_char.to_uppercase().next().unwrap_or(*first_char);
        }
        chars.into_iter().collect()
    }
}

/// Pattern-based title generator (temporary replacement for ML model)
struct PatternMatcher {
    action_patterns: HashMap<String, Vec<String>>,
    domain_patterns: HashMap<String, Vec<String>>,
    cleanup_regex: Vec<Regex>,
}

impl PatternMatcher {
    fn new() -> Result<Self> {
        let mut action_patterns = HashMap::new();
        let mut domain_patterns = HashMap::new();
        
        // Common action patterns
        action_patterns.insert("fix".to_string(), vec![
            "Fix {domain} {issue}".to_string(),
            "Resolve {domain} {issue}".to_string(),
            "Correct {domain} {issue}".to_string(),
        ]);
        
        action_patterns.insert("feature".to_string(), vec![
            "Add {domain} {feature}".to_string(),
            "Implement {domain} {feature}".to_string(),
            "Introduce {domain} {feature}".to_string(),
        ]);
        
        action_patterns.insert("refactor".to_string(), vec![
            "Refactor {domain} {component}".to_string(),
            "Improve {domain} {component}".to_string(),
            "Optimize {domain} {component}".to_string(),
        ]);
        
        // Domain-specific patterns
        domain_patterns.insert("auth".to_string(), vec![
            "authentication".to_string(),
            "authorization".to_string(),
            "login".to_string(),
            "security".to_string(),
        ]);
        
        domain_patterns.insert("crypto".to_string(), vec![
            "cryptocurrency".to_string(),
            "blockchain".to_string(),
            "wallet".to_string(),
        ]);
        
        domain_patterns.insert("api".to_string(), vec![
            "API".to_string(),
            "endpoint".to_string(),
            "service".to_string(),
        ]);
        
        let cleanup_regex = vec![
            Regex::new(r"\b(the|a|an)\b")?,
            Regex::new(r"\s+")?,
        ];
        
        Ok(Self {
            action_patterns,
            domain_patterns,
            cleanup_regex,
        })
    }
    
    fn generate_title(&self, context: &CleanContext, config: &GeneratorConfig) -> Result<String> {
        // Extract key information
        let action = self.determine_action(context);
        let domain = self.extract_domain(context);
        let main_subject = self.extract_main_subject(context);
        
        // Generate title based on patterns
        let title = if let Some(patterns) = self.action_patterns.get(&action) {
            let pattern_index = (config.temperature * patterns.len() as f32) as usize;
            let pattern = patterns.get(pattern_index).unwrap_or(&patterns[0]);
            
            pattern
                .replace("{domain}", &domain)
                .replace("{issue}", &main_subject)
                .replace("{feature}", &main_subject)
                .replace("{component}", &main_subject)
        } else {
            // Fallback to simple pattern
            if domain.is_empty() {
                main_subject
            } else {
                format!("{} {}", self.capitalize_first(&action), main_subject)
            }
        };
        
        Ok(self.clean_title(&title))
    }
    
    fn determine_action(&self, context: &CleanContext) -> String {
        if let Some(change_type) = &context.change_type {
            change_type.as_str().to_string()
        } else {
            // Infer from commits
            let all_text = context.commits.join(" ").to_lowercase();
            
            if all_text.contains("fix") || all_text.contains("bug") || all_text.contains("issue") {
                "fix".to_string()
            } else if all_text.contains("add") || all_text.contains("implement") || all_text.contains("feature") {
                "feature".to_string()
            } else if all_text.contains("refactor") || all_text.contains("improve") {
                "refactor".to_string()
            } else {
                "update".to_string()
            }
        }
    }
    
    fn extract_domain(&self, context: &CleanContext) -> String {
        let all_text = format!(
            "{} {}",
            context.description.as_deref().unwrap_or(""),
            context.commits.join(" ")
        ).to_lowercase();
        
        // Look for domain keywords
        for (key, aliases) in &self.domain_patterns {
            if aliases.iter().any(|alias| all_text.contains(&alias.to_lowercase())) {
                return key.clone();
            }
        }
        
        // Extract first meaningful word
        all_text
            .split_whitespace()
            .find(|word| word.len() > 3 && !word.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or("")
            .to_string()
    }
    
    fn extract_main_subject(&self, context: &CleanContext) -> String {
        // Combine description and commits
        let mut subjects = Vec::new();
        
        if let Some(desc) = &context.description {
            subjects.push(desc.clone());
        }
        
        subjects.extend(context.commits.iter().cloned());
        
        if subjects.is_empty() {
            return "changes".to_string();
        }
        
        // Find the most descriptive subject
        subjects
            .into_iter()
            .max_by_key(|s| s.len())
            .unwrap_or_else(|| "changes".to_string())
    }
    
    fn clean_title(&self, title: &str) -> String {
        let mut clean = title.to_string();
        
        for regex in &self.cleanup_regex {
            clean = regex.replace_all(&clean, " ").to_string();
        }
        
        clean.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    fn capitalize_first(&self, s: &str) -> String {
        let mut chars: Vec<char> = s.chars().collect();
        if let Some(first) = chars.first_mut() {
            *first = first.to_uppercase().next().unwrap_or(*first);
        }
        chars.into_iter().collect()
    }
}

// TODO: Future ML model integration using candle-rs
#[allow(dead_code)]
struct CandeModel {
    // This will hold the actual ML model when implemented
    // model: candle_core::Device,
    // tokenizer: tokenizers::Tokenizer,
}

#[allow(dead_code)]
impl CandeModel {
    // TODO: Implement actual ML model loading and inference
    async fn load_model(_model_name: &str) -> Result<Self> {
        // Implementation will load actual transformer model using candle-rs
        Err(Error::ModelError {
            message: "ML model integration not yet implemented".to_string(),
        })
    }
    
    async fn generate(&self, _prompt: &str, _config: &GeneratorConfig) -> Result<String> {
        // Implementation will perform actual ML inference
        Err(Error::ModelError {
            message: "ML inference not yet implemented".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ChangeType;
    
    #[test]
    fn test_title_generator_creation() {
        let config = GeneratorConfig::default();
        let generator = TitleGenerator::new(config).unwrap();
        assert!(generator.config.temperature > 0.0);
    }
    
    #[test]
    fn test_invalid_temperature() {
        let config = GeneratorConfig::default().with_temperature(2.0);
        let result = TitleGenerator::new(config);
        assert!(matches!(result, Err(Error::InvalidTemperature { .. })));
    }
    
    #[test]
    fn test_unsupported_model() {
        let config = GeneratorConfig::default().with_model("unknown-model");
        let result = TitleGenerator::new(config);
        assert!(matches!(result, Err(Error::UnsupportedModel { .. })));
    }
    
    #[tokio::test]
    async fn test_pattern_based_generation() {
        let config = GeneratorConfig::default();
        let generator = TitleGenerator::new(config).unwrap();
        
        let context = CleanContext {
            ticket: Some("CRU-310".to_string()),
            change_type: Some(ChangeType::Fix),
            description: Some("bottle stuck issue".to_string()),
            commits: vec!["fix bottle stuck with remediation".to_string()],
        };
        
        let title = generator.generate_title(&context).await.unwrap();
        assert!(!title.is_empty());
        assert!(title.len() <= 72);
    }
}