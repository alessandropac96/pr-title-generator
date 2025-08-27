//! Command line interface for the PR title generator

use crate::{GeneratorConfig, Result};
use clap::{Parser, ValueEnum};

/// Generate meaningful PR titles using ML models
#[derive(Parser)]
#[command(name = "generate-pr-title")]
#[command(about = "A machine learning-based PR title generator")]
#[command(version = "1.0.0")]
#[command(long_about = r#"
Generate meaningful PR titles using ML models

This tool analyzes commit messages and branch context to generate
concise, meaningful pull request titles. It supports multiple ML models
and provides various configuration options for customization.

Examples:
  generate-pr-title                        # Generate title for current branch
  generate-pr-title --verbose              # Enable verbose output  
  generate-pr-title --branch feature/auth  # Generate for specific branch
  generate-pr-title --model phi-2          # Use different ML model
  generate-pr-title --temperature 0.5      # Adjust generation creativity
"#)]
pub struct Cli {
    /// Branch to analyze (defaults to current branch)
    #[arg(long)]
    pub branch: Option<String>,

    /// Base branch to compare against
    #[arg(long, default_value = "main")]
    pub base: String,

    /// Maximum number of commits to analyze
    #[arg(long, default_value = "20")]
    pub max_commits: usize,

    /// LLM model to use
    #[arg(long, default_value = "tiny-llama")]
    pub model: ModelType,

    /// Generation temperature (0.1-1.0)
    #[arg(long, default_value = "0.7")]
    pub temperature: f32,

    /// Maximum title length
    #[arg(long, default_value = "50")]
    pub max_length: usize,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

/// Supported ML models
#[derive(Clone, Debug, ValueEnum)]
pub enum ModelType {
    #[value(name = "tiny-llama")]
    TinyLlama,
    #[value(name = "phi-2")]
    Phi2,
    #[value(name = "gemma-2b")]
    Gemma2b,
    #[value(name = "llama-2-7b")]
    Llama2_7b,
}

impl ModelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::TinyLlama => "tiny-llama",
            ModelType::Phi2 => "phi-2",
            ModelType::Gemma2b => "gemma-2b",
            ModelType::Llama2_7b => "llama-2-7b",
        }
    }
}

impl Cli {
    /// Parse command line arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
    
    /// Validate command line arguments
    pub fn validate(&self) -> Result<()> {
        // Validate temperature range
        if self.temperature < 0.1 || self.temperature > 1.0 {
            return Err(crate::Error::InvalidTemperature {
                temp: self.temperature,
            });
        }
        
        // Validate max length
        if self.max_length == 0 {
            return Err(crate::Error::InvalidMaxLength {
                length: self.max_length,
            });
        }
        
        Ok(())
    }
    
    /// Convert CLI arguments to GeneratorConfig
    pub fn to_config(&self) -> GeneratorConfig {
        GeneratorConfig {
            model_name: self.model.as_str().to_string(),
            temperature: self.temperature,
            max_length: self.max_length,
            max_commits: self.max_commits,
            verbose: self.verbose,
        }
    }
    
    /// Get the branch name, using current branch if not specified
    pub fn get_branch_name(&self) -> Result<String> {
        if let Some(branch) = &self.branch {
            Ok(branch.clone())
        } else {
            self.get_current_branch()
        }
    }
    
    /// Get the current git branch
    fn get_current_branch(&self) -> Result<String> {
        use std::process::Command;
        
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .output()
            .map_err(|e| crate::Error::Io(e))?;
        
        if !output.status.success() {
            return Err(crate::Error::NoBranch);
        }
        
        let branch = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();
        
        if branch.is_empty() {
            Err(crate::Error::NoBranch)
        } else {
            Ok(branch)
        }
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            branch: None,
            base: "main".to_string(),
            max_commits: 20,
            model: ModelType::TinyLlama,
            temperature: 0.7,
            max_length: 50,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_type_conversion() {
        assert_eq!(ModelType::TinyLlama.as_str(), "tiny-llama");
        assert_eq!(ModelType::Phi2.as_str(), "phi-2");
        assert_eq!(ModelType::Gemma2b.as_str(), "gemma-2b");
        assert_eq!(ModelType::Llama2_7b.as_str(), "llama-2-7b");
    }
    
    #[test]
    fn test_config_conversion() {
        let cli = Cli {
            model: ModelType::Phi2,
            temperature: 0.5,
            max_length: 60,
            max_commits: 30,
            verbose: true,
            ..Default::default()
        };
        
        let config = cli.to_config();
        assert_eq!(config.model_name, "phi-2");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_length, 60);
        assert_eq!(config.max_commits, 30);
        assert!(config.verbose);
    }
    
    #[test]
    fn test_temperature_validation() {
        let cli = Cli {
            temperature: 2.0,
            ..Default::default()
        };
        
        assert!(cli.validate().is_err());
        
        let cli = Cli {
            temperature: 0.5,
            ..Default::default()
        };
        
        assert!(cli.validate().is_ok());
    }
    
    #[test]
    fn test_max_length_validation() {
        let cli = Cli {
            max_length: 0,
            ..Default::default()
        };
        
        assert!(cli.validate().is_err());
        
        let cli = Cli {
            max_length: 50,
            ..Default::default()
        };
        
        assert!(cli.validate().is_ok());
    }
}