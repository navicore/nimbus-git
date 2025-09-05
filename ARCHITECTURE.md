# Nimbus Git - Cloud-Native Git Server

## Vision
A cloud-native git platform built in Rust that serves as an experimentation framework for next-generation development workflows. Core git functionality with pluggable extensions for CI/CD, code review, and AI collaboration.

## Core Philosophy

### "Minimal Core, Maximum Extensibility"
- The core provides rock-solid git operations
- Everything else is a plugin
- Users bring their own tools (CI runners, AI models, review systems)
- The platform provides the glue and coordination

### "Single Owner, Multiple Collaborators"
**This is NOT a GitHub clone.** Each instance has:
- **One Owner**: The person who runs the instance owns all repos
- **Many Collaborators**: Friends, colleagues who can contribute
- **No User Repos**: Collaborators can't create their own repos
- **No Forking**: Want to fork? Clone it to YOUR instance

**Why This Matters**:
```
Traditional (GitHub/GitLab):
github.com/alice/project   <- Alice's namespace
github.com/bob/project     <- Bob's namespace  
github.com/alice/project/fork/bob  <- Confusing!

Our Model:
code.alice.dev/project     <- Alice's instance
code.bob.dev/project       <- Bob's instance (his fork)
code.corp.com/project      <- Company instance
```

**Benefits**:
1. **Clear Ownership**: Your instance, your rules, your resources
2. **Simple Permissions**: Read/Write/Admin per repo, that's it
3. **Natural Federation**: Each instance is sovereign
4. **No Social Clutter**: No stars, follows, trending - just code

## Core Principles
1. **Cloud-Native First** - Every decision optimized for K8s deployment
2. **Stateless Pods** - All state externalized to appropriate stores
3. **Configuration as Code** - No files, only ConfigMaps/Secrets/Environment
4. **Type Safety** - Rust everywhere, shared types frontend to backend
5. **Progressive Enhancement** - Works without JS, enhanced with WASM
6. **Plugin Architecture** - Core features are minimal, innovation happens in plugins
7. **Event-Driven** - Everything emits events that plugins can subscribe to

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                   Cloudflare Tunnel                  │
│                  (HTTPS Traffic Only)                │
└─────────────────┬────────────────────────────────────┘
                  │
┌─────────────────▼────────────────────────────────────┐
│              Kubernetes Ingress                      │
│         (Future: TCP for SSH support)                │
└─────────────────┬────────────────────────────────────┘
                  │
        ┌─────────┴─────────┬─────────────┐
        │                   │             │
┌───────▼────────┐ ┌────────▼──────┐ ┌───▼────────────┐
│  Web Service   │ │  Git Service  │ │  Auth Service  │
│   (Leptos)     │ │    (Warp)     │ │   (Warp)       │
│                │ │               │ │                │
│  - UI Serving  │ │ - Git HTTP   │ │ - JWT Issue    │
│  - WASM Assets │ │ - Git SSH*   │ │ - OIDC/OAuth   │
│  - SSR         │ │ - Webhooks   │ │ - API Keys     │
└────────────────┘ └───────────────┘ └────────────────┘
        │                   │             │
        └─────────┬─────────┴─────────────┘
                  │
        ┌─────────▼─────────┬─────────────┐
        │                   │             │
┌───────▼────────┐ ┌────────▼──────┐ ┌───▼────────────┐
│     Redis      │ │  PostgreSQL   │ │ S3-Compatible  │
│                │ │               │ │   Storage      │
│  - Sessions    │ │ - Metadata    │ │ - Git Objects  │
│  - Cache       │ │ - Users       │ │ - LFS Files    │
│  - Pub/Sub     │ │ - Repos       │ │ - Artifacts    │
└────────────────┘ └───────────────┘ └────────────────┘
```

## Component Design

### Shared Types Crate (`nimbus-types`)
- Repository, Commit, User, Branch structs
- API contracts
- Serialization/deserialization
- Validation rules

### Backend Services

#### Git Service (`nimbus-git`)
- **Framework**: Warp
- **Git Library**: gitoxide (pure Rust) or git2 (libgit2 bindings)
- **Responsibilities**:
  - Git HTTP smart protocol
  - Git SSH protocol (future)
  - Repository management
  - Commit operations
  - Branch/tag management
  - Webhooks

#### Web Service (`nimbus-web`)
- **Framework**: Leptos with SSR
- **WASM Components**:
  - Repository browser
  - Commit viewer with diff
  - Branch management
  - User settings
  - Admin panel
- **Features**:
  - Progressive enhancement
  - Offline-capable with service workers
  - Real-time updates via WebSockets

#### Auth Service (`nimbus-auth`)
- **Framework**: Warp
- **Features**:
  - JWT token issuance
  - OAuth2/OIDC provider support
  - API key management
  - RBAC with K8s integration
  - Session management via Redis

### Storage Strategy

#### Git Objects
- **Primary**: S3-compatible storage (MinIO, Ceph, AWS S3)
- **Cache**: Local disk with LRU eviction
- **Alternative**: Distributed filesystem (GlusterFS)

#### Metadata
- **PostgreSQL** for:
  - Repository metadata
  - User accounts
  - Permissions
  - Audit logs
  - CI/CD configurations

#### Sessions & Cache
- **Redis** for:
  - User sessions
  - Auth tokens
  - Git object cache
  - Real-time pub/sub

## Cloud-Native Features

### Kubernetes Native
- **ConfigMaps** for all configuration
- **Secrets** for credentials
- **Service Mesh** ready (Istio/Linkerd)
- **Horizontal Pod Autoscaling**
- **Pod Disruption Budgets**
- **Liveness/Readiness probes**
- **Operator pattern** for complex operations

### Observability
- **Metrics**: Prometheus metrics
- **Tracing**: OpenTelemetry
- **Logging**: Structured JSON logs
- **Health checks**: Deep health endpoints

### Security
- **Zero Trust**: mTLS between services
- **RBAC**: Kubernetes-native
- **Secrets Management**: Vault integration
- **Policy as Code**: OPA for authorization

## Deployment Modes

### 1. Single Binary (Development)
- All services in one binary
- SQLite for metadata
- Local filesystem for git objects
- In-memory session store

### 2. Kubernetes (Production)
- Separate deployments per service
- PostgreSQL StatefulSet or RDS
- S3 for git objects
- Redis for sessions

### 3. Edge (Cloudflare Workers)
- WASM-compiled git operations
- Durable Objects for state
- R2 for storage

## Git Protocol Support

### Phase 1: HTTPS Only
- Git HTTP smart protocol
- Basic auth / Bearer tokens
- Works through Cloudflare tunnel

### Phase 2: SSH Support
- SSH protocol implementation
- Requires direct ingress or NodePort
- Certificate-based auth option

### Phase 3: Git Protocol
- Native git:// protocol
- Read-only anonymous access
- Performance optimized

## Plugin Architecture

### Core Extension Points

The platform provides three primary extension systems:

#### 1. CI/CD Runners (Pluggable)
**Interface**: Event-based triggers + Runner API
```rust
trait CIRunner {
    async fn handle_push(&self, event: PushEvent) -> RunResult;
    async fn handle_pr(&self, event: PREvent) -> RunResult;
    async fn get_status(&self, run_id: Uuid) -> RunStatus;
}
```
**Implementations**:
- Kubernetes Job runner
- GitHub Actions bridge
- Buildkite/CircleCI bridge
- Local Docker runner
- WASM sandbox runner

#### 2. Code Review System (Pluggable)
**Interface**: Review API + UI Components
```rust
trait ReviewSystem {
    async fn create_review(&self, pr: PullRequest) -> Review;
    async fn add_comment(&self, review: Review, comment: Comment);
    async fn suggest_change(&self, diff: Diff) -> Vec<Suggestion>;
}
```
**Implementations**:
- Basic PR comments
- Reviewable.io-style interface
- Gerrit-style workflow
- Pair programming mode

#### 3. AI Participation (MCP-based)
**Interface**: Model Context Protocol
```rust
trait AIReviewer {
    async fn analyze_diff(&self, diff: Diff) -> Analysis;
    async fn suggest_improvements(&self, code: Code) -> Vec<Suggestion>;
    async fn answer_question(&self, context: Context, q: String) -> String;
}
```
**Implementations**:
- Claude via MCP
- GPT-4 via MCP
- Local Ollama models
- Specialized code models (CodeLlama, etc.)

### Event Bus Architecture

All plugins communicate via events:

```
Git Operation → Event Bus → Plugin Subscriptions
                    ↓
              [PushEvent]
                ↙   ↓   ↘
          CI/CD  Review  AI
```

**Core Events**:
- `RepositoryCreated`
- `BranchCreated/Deleted`
- `CommitPushed`
- `PullRequestOpened/Merged`
- `TagCreated`
- `FileChanged`

### Plugin Management

#### Discovery & Registration
- Plugins register via gRPC or HTTP endpoints
- Service mesh integration for plugin discovery
- Health checks and circuit breakers

#### Configuration
```yaml
apiVersion: nimbus.git/v1
kind: Plugin
metadata:
  name: github-actions-runner
spec:
  type: ci-runner
  endpoint: grpc://actions-runner:50051
  config:
    github_token: ${SECRET_GITHUB_TOKEN}
```

#### Security
- Plugins run in separate containers/pods
- mTLS between core and plugins
- RBAC for plugin permissions
- Resource quotas per plugin

### Extension Philosophy

**What the Core Does**:
- Git operations (clone, push, pull)
- User authentication
- Repository management
- Event emission
- Plugin coordination

**What Plugins Do**:
- CI/CD execution
- Code review workflows
- AI analysis
- Custom workflows
- Integration with external tools

This separation ensures:
1. Core remains simple and maintainable
2. Experimentation doesn't destabilize core
3. Users can bring their own tools
4. Multiple implementations can coexist

## Roadmap

### MVP (Phase 1) - 2-3 months
- [ ] Basic Rust workspace setup
- [ ] Git HTTP protocol (clone/push/pull)
- [ ] Simple web UI (repository list, browse)
- [ ] Basic auth (username/password)
- [ ] PostgreSQL metadata store
- [ ] Local file storage
- [ ] Docker container
- [ ] Basic K8s manifests

### Beta (Phase 2) - 3-6 months
- [ ] S3-compatible storage
- [ ] Redis sessions
- [ ] OAuth2/OIDC support
- [ ] WASM-enhanced UI
- [ ] Webhook support
- [ ] Helm chart
- [ ] Horizontal scaling

### Production (Phase 3) - 6-12 months
- [ ] SSH protocol support
- [ ] Advanced UI (PR/MR workflow)
- [ ] CI/CD integration
- [ ] Federation support
- [ ] Kubernetes operator
- [ ] Multi-tenancy

## Success Criteria
1. **Cloud-Native**: Runs perfectly in Kubernetes with horizontal scaling
2. **Performance**: Faster than Gitea for common operations
3. **Simple**: Less configuration than GitLab
4. **Modern**: WASM UI, Rust performance
5. **Extensible**: Plugin system for custom workflows