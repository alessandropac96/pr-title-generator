//! Error types for the PR title generator

use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Not a git repository: {path}")]
    NotGitRepository { path: PathBuf },
    
    #[error("Could not determine current branch")]
    NoBranch,
    
    #[error("Branch '{branch}' not found")]
    BranchNotFound { branch: String },
    
    #[error("Base branch '{branch}' not found")]
    BaseBranchNotFound { branch: String },
    
    #[error("No commits found between '{base}' and '{branch}'")]
    NoCommits { base: String, branch: String },
    
    #[error("ML model error: {message}")]
    ModelError { message: String },
    
    #[error("Model '{name}' not supported")]
    UnsupportedModel { name: String },
    
    #[error("Invalid temperature: {temp}. Must be between 0.1 and 1.0")]
    InvalidTemperature { temp: f32 },
    
    #[error("Invalid max length: {length}. Must be greater than 0")]
    InvalidMaxLength { length: usize },
}