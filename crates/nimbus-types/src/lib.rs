//! Shared types for Nimbus Git platform
//! 
//! Single-owner model: Each instance has one owner and multiple collaborators.
//! This is NOT a GitHub clone - it's a personal git platform.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod events;

/// The instance owner - there's only one per deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub username: String,
    pub email: String,
    pub instance_domain: String, // e.g., "code.navicore.tech"
}

/// Collaborators can contribute but not create repos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub ssh_keys: Vec<SshKey>,
    pub api_tokens: Vec<ApiToken>,
}

/// Simple permission model - no complex RBAC needed
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Admin,
}

/// Repository belongs to the instance owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_private: bool,
    pub default_branch: String,
    pub collaborator_permissions: Vec<CollaboratorPermission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorPermission {
    pub collaborator_id: Uuid,
    pub repository_id: Uuid,
    pub permission: Permission,
}

/// SSH key for git operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub id: Uuid,
    pub name: String,
    pub public_key: String,
    pub fingerprint: String,
}

/// API token for HTTPS git and API access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: Uuid,
    pub name: String,
    pub token_hash: String, // Store hash, not plaintext
    pub expires_at: Option<time::OffsetDateTime>,
}

/// Events that plugins can subscribe to
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    RepositoryCreated {
        repository: Repository,
    },
    RepositoryDeleted {
        repository_id: Uuid,
    },
    Push {
        repository_id: Uuid,
        branch: String,
        commits: Vec<Commit>,
        pusher: String,
    },
    PullRequestOpened {
        id: Uuid,
        repository_id: Uuid,
        from_branch: String,
        to_branch: String,
        author: String,
    },
    PullRequestMerged {
        id: Uuid,
        repository_id: Uuid,
    },
    TagCreated {
        repository_id: Uuid,
        tag: String,
        target_commit: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub timestamp: time::OffsetDateTime,
    pub parent_shas: Vec<String>,
}

/// Plugin types for the extension system
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PluginType {
    CiRunner,
    ReviewSystem,
    AiReviewer,
}

/// Plugin registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: Uuid,
    pub name: String,
    pub plugin_type: PluginType,
    pub endpoint: String, // gRPC or HTTP endpoint
    pub health_check: String,
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum NimbusError {
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Invalid git operation: {0}")]
    InvalidGitOperation(String),
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}