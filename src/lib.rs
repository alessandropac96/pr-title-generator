//! PR Title Generator Library
//! 
//! A machine learning-based library for generating meaningful PR titles
//! from commit messages and branch context.

pub mod cli;
pub mod git;
pub mod context;
pub mod ml;
pub mod error;

pub use error::{Error, Result};

/// Configuration for the PR title generator
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub model_name: String,
    pub temperature: f32,
    pub max_length: usize,
    pub max_commits: usize,
    pub verbose: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            model_name: "tiny-llama".to_string(),
            temperature: 0.7,
            max_length: 50,
            max_commits: 20,
            verbose: false,
        }
    }
}

impl GeneratorConfig {
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model_name = model.into();
        self
    }
    
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
    
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = max_length;
        self
    }
    
    pub fn with_max_commits(mut self, max_commits: usize) -> Self {
        self.max_commits = max_commits;
        self
    }
    
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}