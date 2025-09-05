# Nimbus Git REST API

Clean, simple API reflecting our single-owner model. No user namespaces, no complex permissions.

## Authentication

```
Authorization: Bearer <token>
```
or
```
X-API-Key: <api-key>
```

## Core Endpoints

### Instance Info

```http
GET /api/v1/instance
```
```json
{
  "owner": "navicore",
  "domain": "code.navicore.tech",
  "version": "0.1.0",
  "features": ["ci_runner", "code_review", "ai_review"]
}
```

### Repositories

#### List all repositories
```http
GET /api/v1/repos
```
```json
[
  {
    "name": "nimbus-git",
    "description": "Cloud-native git platform",
    "is_private": false,
    "default_branch": "main",
    "clone_urls": {
      "ssh": "git@code.navicore.tech:nimbus-git.git",
      "https": "https://code.navicore.tech/nimbus-git.git"
    }
  }
]
```

#### Get repository
```http
GET /api/v1/repos/{name}
```

#### Create repository (owner only)
```http
POST /api/v1/repos
```
```json
{
  "name": "new-project",
  "description": "My new project",
  "is_private": false,
  "default_branch": "main"
}
```

#### Delete repository (owner only)
```http
DELETE /api/v1/repos/{name}
```

### Collaborators

#### List collaborators (owner only)
```http
GET /api/v1/collaborators
```
```json
[
  {
    "username": "alice",
    "email": "alice@example.com",
    "permissions": {
      "nimbus-git": "write",
      "secret-project": "admin"
    }
  }
]
```

#### Add collaborator (owner only)
```http
POST /api/v1/collaborators
```
```json
{
  "username": "bob",
  "email": "bob@example.com",
  "ssh_key": "ssh-rsa AAAAB3..."
}
```

#### Update permissions (owner only)
```http
PUT /api/v1/repos/{name}/permissions
```
```json
{
  "collaborator": "alice",
  "permission": "admin"  // read | write | admin
}
```

### Git Operations

#### Get commits
```http
GET /api/v1/repos/{name}/commits?branch={branch}&limit={limit}
```

#### Get file content
```http
GET /api/v1/repos/{name}/content/{path}?ref={branch|tag|sha}
```

#### Get diff
```http
GET /api/v1/repos/{name}/diff/{from}...{to}
```

### Pull Requests (Simple)

#### List PRs
```http
GET /api/v1/repos/{name}/pulls
```

#### Create PR
```http
POST /api/v1/repos/{name}/pulls
```
```json
{
  "title": "Add new feature",
  "from_branch": "feature-x",
  "to_branch": "main",
  "description": "This adds feature X"
}
```

#### Merge PR
```http
POST /api/v1/repos/{name}/pulls/{id}/merge
```
```json
{
  "method": "merge" // merge | squash | rebase
}
```

### Events (WebSocket)

```http
WS /api/v1/events
```
```json
{
  "subscribe": ["push", "pull_request", "tag"]
}
```
Receives:
```json
{
  "type": "push",
  "repository": "nimbus-git",
  "branch": "main",
  "commits": [...]
}
```

### Plugins

#### List plugins
```http
GET /api/v1/plugins
```
```json
[
  {
    "name": "github-actions-runner",
    "type": "ci_runner",
    "status": "healthy",
    "endpoint": "grpc://localhost:50051"
  }
]
```

#### Register plugin (owner only)
```http
POST /api/v1/plugins
```
```json
{
  "name": "my-ci-runner",
  "type": "ci_runner",
  "endpoint": "https://ci.example.com/webhook"
}
```

#### Plugin webhook
```http
POST /api/v1/plugins/{name}/webhook
```
Plugin-specific payload

### CI/CD (via plugins)

#### Get runs
```http
GET /api/v1/repos/{name}/runs
```

#### Get run details
```http
GET /api/v1/repos/{name}/runs/{id}
```

#### Trigger run
```http
POST /api/v1/repos/{name}/runs
```
```json
{
  "branch": "main",
  "plugin": "github-actions-runner"
}
```

### Code Review (via plugins)

#### Get reviews
```http
GET /api/v1/repos/{name}/pulls/{id}/reviews
```

#### Create review
```http
POST /api/v1/repos/{name}/pulls/{id}/reviews
```
```json
{
  "plugin": "reviewable-plugin",
  "action": "request_changes",
  "comments": [...]
}
```

### AI Integration (via plugins)

#### Ask AI
```http
POST /api/v1/repos/{name}/ai/ask
```
```json
{
  "plugin": "claude-mcp",
  "context": {
    "file": "src/main.rs",
    "line": 42
  },
  "question": "How can I optimize this function?"
}
```

#### AI Review
```http
POST /api/v1/repos/{name}/pulls/{id}/ai-review
```
```json
{
  "plugin": "gpt4-reviewer",
  "focus": ["security", "performance"]
}
```

## Git HTTP Protocol

Standard Git HTTP protocol at:

```
GET /git/{repo}.git/info/refs?service=git-upload-pack
POST /git/{repo}.git/git-upload-pack
GET /git/{repo}.git/info/refs?service=git-receive-pack
POST /git/{repo}.git/git-receive-pack
```

## Design Notes

### What's NOT in this API:
- No `/users` endpoints - single owner model
- No `/orgs` or `/teams` - not needed
- No social features (stars, watches, follows)
- No forking endpoints - fork to your own instance
- No complex permissions - just read/write/admin

### Clean URLs:
- `/api/v1/repos/nimbus-git` ✓
- NOT `/api/v1/users/navicore/repos/nimbus-git` ✗

### Plugin Philosophy:
- Core API is minimal
- Plugins extend via webhooks and subscriptions
- Multiple plugins of same type can coexist
- Plugins are isolated - can't break core

This API is probably 1/10th the size of GitHub's, yet covers all essential operations!