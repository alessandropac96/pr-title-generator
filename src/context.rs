//! Branch context extraction and text processing

use crate::{git::CommitInfo, Result};
use regex::Regex;
use std::collections::HashSet;

/// Extracted context from a branch name and commits
#[derive(Debug, Clone)]
pub struct BranchContext {
    pub ticket: Option<String>,
    pub change_type: Option<ChangeType>,
    pub description: Option<String>,
}

/// Type of change inferred from branch name or commits
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Fix,
    Feature,
    Refactor,
    Hotfix,
    Chore,
    Docs,
}

impl ChangeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Fix => "fix",
            ChangeType::Feature => "feature",
            ChangeType::Refactor => "refactor",
            ChangeType::Hotfix => "hotfix",
            ChangeType::Chore => "chore",
            ChangeType::Docs => "docs",
        }
    }
}

/// Context processor for extracting meaningful information from git data
pub struct ContextProcessor {
    // Precompiled regex patterns for efficiency
    ticket_regex: Regex,
    noise_patterns: Vec<Regex>,
    generic_terms: HashSet<String>,
}

impl ContextProcessor {
    pub fn new() -> Result<Self> {
        let ticket_regex = Regex::new(r"([A-Z]+-\d+)")?;
        
        let noise_patterns = vec![
            Regex::new(r"\b\d{4,}\b")?,                    // Long numbers
            Regex::new(r"\b[a-f0-9]{8,}\b")?,             // Hex strings
            Regex::new(r"\b(cursor|origin|main|master|develop)\b")?, // Branch prefixes
            Regex::new(r"\b(update|update-|update_)\b")?,   // Generic update prefixes
            Regex::new(r"\s+")?,                           // Multiple spaces
        ];
        
        let generic_terms = ["update", "change", "modify", "fix", "improve", "add", "remove"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        Ok(Self {
            ticket_regex,
            noise_patterns,
            generic_terms,
        })
    }
    
    /// Extract context from a branch name
    pub fn extract_branch_context(&self, branch_name: &str) -> BranchContext {
        let clean_branch = self.remove_branch_prefixes(branch_name);
        let ticket = self.extract_ticket_number(&clean_branch);
        let change_type = self.infer_change_type(&clean_branch);
        let description = self.extract_description(&clean_branch, &ticket);
        
        BranchContext {
            ticket,
            change_type,
            description,
        }
    }
    
    /// Clean commit messages by removing noise and redundant information
    pub fn clean_commit_messages(&self, commits: &[CommitInfo]) -> Vec<String> {
        commits
            .iter()
            .filter_map(|commit| self.clean_single_commit_message(commit.clean_message()))
            .collect()
    }
    
    /// Create a cleaned context for ML model input
    pub fn create_clean_context(
        &self,
        branch_context: &BranchContext,
        commit_messages: &[String],
    ) -> CleanContext {
        let meaningful_commits = self.filter_meaningful_commits(commit_messages);
        
        CleanContext {
            ticket: branch_context.ticket.clone(),
            change_type: branch_context.change_type.clone(),
            description: branch_context.description.clone(),
            commits: meaningful_commits,
        }
    }
    
    /// Remove common branch prefixes
    fn remove_branch_prefixes(&self, branch_name: &str) -> String {
        branch_name
            .replace("origin/", "")
            .replace("cursor/", "")
            .replace("refs/heads/", "")
            .replace("refs/remotes/", "")
    }
    
    /// Extract ticket number from branch name
    fn extract_ticket_number(&self, branch_name: &str) -> Option<String> {
        self.ticket_regex
            .find(branch_name)
            .map(|m| m.as_str().to_string())
            .filter(|ticket| self.is_meaningful_ticket(ticket))
    }
    
    /// Check if a ticket number looks meaningful (not just random numbers)
    fn is_meaningful_ticket(&self, ticket: &str) -> bool {
        let prefixes = ["CRU-", "JIRA-", "TASK-", "BUG-", "FEATURE-", "FIX-"];
        prefixes.iter().any(|prefix| ticket.starts_with(prefix))
    }
    
    /// Infer the type of change from branch name
    fn infer_change_type(&self, branch_name: &str) -> Option<ChangeType> {
        let lower_branch = branch_name.to_lowercase();
        
        if lower_branch.contains("hotfix") {
            Some(ChangeType::Hotfix)
        } else if lower_branch.contains("fix") || lower_branch.contains("bug") {
            Some(ChangeType::Fix)
        } else if lower_branch.contains("feature") || lower_branch.contains("feat") {
            Some(ChangeType::Feature)
        } else if lower_branch.contains("refactor") {
            Some(ChangeType::Refactor)
        } else if lower_branch.contains("docs") || lower_branch.contains("doc") {
            Some(ChangeType::Docs)
        } else if lower_branch.contains("chore") {
            Some(ChangeType::Chore)
        } else {
            None
        }
    }
    
    /// Extract description from branch name
    fn extract_description(&self, branch_name: &str, ticket: &Option<String>) -> Option<String> {
        let words: Vec<&str> = branch_name.split(&['-', '_', '/']).collect();
        
        if words.len() <= 2 {
            return None;
        }
        
        let start_idx = if ticket.is_some() { 2 } else { 1 };
        
        if start_idx >= words.len() {
            return None;
        }
        
        let description_words: Vec<&str> = words[start_idx..].iter().copied().collect();
        let description = description_words.join(" ");
        let clean_description = self.clean_text(&description);
        
        if clean_description.len() > 3 && !clean_description.chars().all(|c| c.is_ascii_digit()) {
            Some(clean_description)
        } else {
            None
        }
    }
    
    /// Clean a single commit message
    fn clean_single_commit_message(&self, message: &str) -> Option<String> {
        let mut clean_message = message.to_string();
        
        // Remove conventional commit prefixes
        let conventional_prefixes = [
            "fix:", "feat:", "feature:", "bug:", "hotfix:", "refactor:",
            "docs:", "style:", "test:", "chore:", "perf:", "ci:",
        ];
        
        for prefix in &conventional_prefixes {
            if clean_message.to_lowercase().starts_with(prefix) {
                clean_message = clean_message[prefix.len()..].trim().to_string();
                break;
            }
        }
        
        // Remove merge and revert messages
        if clean_message.to_lowercase().contains("merge") 
            || clean_message.to_lowercase().contains("revert") {
            return None;
        }
        
        let cleaned = self.clean_text(&clean_message);
        
        if cleaned.len() > 5 {
            Some(cleaned)
        } else {
            None
        }
    }
    
    /// Clean text by removing noise patterns
    fn clean_text(&self, text: &str) -> String {
        let mut clean_text = text.to_string();
        
        for pattern in &self.noise_patterns {
            clean_text = pattern.replace_all(&clean_text, " ").to_string();
        }
        
        // Clean up whitespace and return
        clean_text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    
    /// Filter commits to keep only meaningful ones
    fn filter_meaningful_commits(&self, commits: &[String]) -> Vec<String> {
        commits
            .iter()
            .filter(|commit| !self.is_generic_commit(commit))
            .cloned()
            .collect()
    }
    
    /// Check if a commit message is too generic to be useful
    fn is_generic_commit(&self, commit: &str) -> bool {
        let words: Vec<&str> = commit.split_whitespace().collect();
        
        // Too short
        if words.len() <= 2 {
            return true;
        }
        
        // Only contains generic terms
        words.iter().all(|word| {
            self.generic_terms.contains(&word.to_lowercase()) || word.len() <= 3
        })
    }
}

/// Cleaned context ready for ML model input
#[derive(Debug, Clone)]
pub struct CleanContext {
    pub ticket: Option<String>,
    pub change_type: Option<ChangeType>,
    pub description: Option<String>,
    pub commits: Vec<String>,
}

impl CleanContext {
    /// Generate a prompt for the ML model
    pub fn to_prompt(&self) -> String {
        let mut context_parts = Vec::new();
        
        if let Some(ticket) = &self.ticket {
            context_parts.push(format!("Ticket: {}", ticket));
        }
        
        if let Some(change_type) = &self.change_type {
            context_parts.push(format!("Type: {}", change_type.as_str()));
        }
        
        if let Some(description) = &self.description {
            context_parts.push(format!("Description: {}", description));
        }
        
        let context_str = if context_parts.is_empty() {
            "No specific context".to_string()
        } else {
            context_parts.join("; ")
        };
        
        let commits_str = if self.commits.is_empty() {
            "No specific changes".to_string()
        } else {
            self.commits.join("; ")
        };
        
        format!(
            "<|system|>
You are a helpful assistant that generates concise, meaningful PR titles based on commit messages and branch context.

TITLE GENERATION RULES:
- Generate a single, clear PR title that summarizes the main changes
- Make it specific to the actual changes
- Focus on what was accomplished, not how it was implemented
- Do not include any explanations or additional text - only the title
- Prioritize user-facing impact over technical implementation details
- Keep it under 72 characters
- Use present tense and active voice

Context: {}
Changes: {}

Generate a concise PR title:<|user|>
Based on the context and changes above, generate a concise PR title that captures the main accomplishment.<|assistant|>",
            context_str, commits_str
        )
    }
}

impl Default for ContextProcessor {
    fn default() -> Self {
        Self::new().expect("Failed to create ContextProcessor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_ticket_number() {
        let processor = ContextProcessor::new().unwrap();
        
        assert_eq!(
            processor.extract_ticket_number("feature/CRU-310-fix-bottle-stuck"),
            Some("CRU-310".to_string())
        );
        
        assert_eq!(
            processor.extract_ticket_number("fix/JIRA-123-update-auth"),
            Some("JIRA-123".to_string())
        );
        
        // Should not match random numbers
        assert_eq!(
            processor.extract_ticket_number("feature/123-some-feature"),
            None
        );
    }
    
    #[test]
    fn test_infer_change_type() {
        let processor = ContextProcessor::new().unwrap();
        
        assert_eq!(
            processor.infer_change_type("fix/bottle-stuck-issue"),
            Some(ChangeType::Fix)
        );
        
        assert_eq!(
            processor.infer_change_type("feature/new-auth-system"),
            Some(ChangeType::Feature)
        );
        
        assert_eq!(
            processor.infer_change_type("hotfix/critical-security-patch"),
            Some(ChangeType::Hotfix)
        );
    }
    
    #[test]
    fn test_clean_commit_message() {
        let processor = ContextProcessor::new().unwrap();
        
        assert_eq!(
            processor.clean_single_commit_message("fix: bottle stuck with remediation system"),
            Some("bottle stuck with remediation system".to_string())
        );
        
        assert_eq!(
            processor.clean_single_commit_message("feat: implement new authentication"),
            Some("implement new authentication".to_string())
        );
        
        // Should filter out merge messages
        assert_eq!(
            processor.clean_single_commit_message("Merge branch 'main' into feature"),
            None
        );
    }
}