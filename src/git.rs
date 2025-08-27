//! Git repository operations and validation

use crate::{Error, Result};
use git2::{Repository, Commit, Oid};
use std::path::{Path, PathBuf};

/// Git repository wrapper with validation and operations
pub struct GitRepo {
    repo: Repository,
    root_path: PathBuf,
}

impl GitRepo {
    /// Open and validate a git repository at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Try to open the git repository - git2 will walk up to find .git
        let repo = Repository::open(path)
            .map_err(|_| Error::NotGitRepository { 
                path: path.to_path_buf() 
            })?;
        let root_path = repo.workdir()
            .ok_or_else(|| Error::NotGitRepository { 
                path: path.to_path_buf() 
            })?
            .to_path_buf();
        
        Ok(Self { repo, root_path })
    }
    
    /// Get the root path of the repository
    pub fn root_path(&self) -> &Path {
        &self.root_path
    }
    
    /// Get the current branch name
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        
        if let Some(name) = head.shorthand() {
            Ok(name.to_string())
        } else {
            Err(Error::NoBranch)
        }
    }
    
    /// Check if a branch exists
    pub fn branch_exists(&self, branch_name: &str) -> bool {
        self.repo.find_branch(branch_name, git2::BranchType::Local).is_ok() ||
        self.repo.find_branch(branch_name, git2::BranchType::Remote).is_ok()
    }
    
    /// Get commits between base and branch
    pub fn get_commits_between(&self, base: &str, branch: &str, max_commits: usize) -> Result<Vec<CommitInfo>> {
        // Resolve branch references
        let branch_oid = self.resolve_reference(branch)?;
        let base_oid = self.resolve_reference(base)?;
        
        // Find merge base (common ancestor)
        let merge_base = self.repo.merge_base(base_oid, branch_oid)?;
        
        // Walk from branch to merge base
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(branch_oid)?;
        revwalk.hide(merge_base)?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::TIME)?;
        
        let mut commits = Vec::new();
        
        for (i, oid) in revwalk.enumerate() {
            if i >= max_commits {
                break;
            }
            
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            
            // Skip merge commits
            if commit.parent_count() > 1 {
                continue;
            }
            
            commits.push(CommitInfo::from_commit(&commit));
        }
        
        if commits.is_empty() {
            return Err(Error::NoCommits {
                base: base.to_string(),
                branch: branch.to_string(),
            });
        }
        
        Ok(commits)
    }
    
    /// Resolve a reference (branch name) to an OID
    fn resolve_reference(&self, reference: &str) -> Result<Oid> {
        // Try as a direct reference first
        if let Ok(reference) = self.repo.find_reference(reference) {
            return Ok(reference.target().unwrap_or_else(|| {
                reference.symbolic_target_bytes()
                    .and_then(|name| self.repo.find_reference(std::str::from_utf8(name).ok()?).ok())
                    .and_then(|r| r.target())
                    .unwrap()
            }));
        }
        
        // Try as a branch name
        if let Ok(branch) = self.repo.find_branch(reference, git2::BranchType::Local) {
            if let Some(oid) = branch.get().target() {
                return Ok(oid);
            }
        }
        
        // Try as a remote branch
        if let Ok(branch) = self.repo.find_branch(reference, git2::BranchType::Remote) {
            if let Some(oid) = branch.get().target() {
                return Ok(oid);
            }
        }
        
        // Try with refs/heads/ prefix
        let full_ref = format!("refs/heads/{}", reference);
        if let Ok(reference) = self.repo.find_reference(&full_ref) {
            if let Some(oid) = reference.target() {
                return Ok(oid);
            }
        }
        
        // Try with refs/remotes/origin/ prefix
        let remote_ref = format!("refs/remotes/origin/{}", reference);
        if let Ok(reference) = self.repo.find_reference(&remote_ref) {
            if let Some(oid) = reference.target() {
                return Ok(oid);
            }
        }
        
        Err(Error::BranchNotFound {
            branch: reference.to_string(),
        })
    }
}

/// Information about a single commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}

impl CommitInfo {
    fn from_commit(commit: &Commit) -> Self {
        Self {
            hash: commit.id().to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("Unknown").to_string(),
            timestamp: commit.time().seconds(),
        }
    }
    
    /// Get the commit message without the hash prefix
    pub fn clean_message(&self) -> &str {
        self.message.trim()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::process::Command;
    
    fn create_test_repo() -> (TempDir, GitRepo) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        
        // Configure git
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        
        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        
        // Create initial commit
        std::fs::write(repo_path.join("README.md"), "# Test Repo").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        
        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        
        let git_repo = GitRepo::open(repo_path).unwrap();
        (temp_dir, git_repo)
    }
    
    #[test]
    fn test_open_git_repo() {
        let (_temp_dir, repo) = create_test_repo();
        assert!(repo.root_path().exists());
    }
    
    #[test]
    fn test_current_branch() {
        let (_temp_dir, repo) = create_test_repo();
        let branch = repo.current_branch().unwrap();
        // Default branch could be "main" or "master"
        assert!(branch == "main" || branch == "master");
    }
    
    #[test]
    fn test_not_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let result = GitRepo::open(temp_dir.path());
        assert!(matches!(result, Err(Error::NotGitRepository { .. })));
    }
}