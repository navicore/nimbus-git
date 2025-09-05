# Nimbus - A Cloud-Native Git Platform

> **Your code, your cloud, your rules.**

Nimbus is a lightweight, extensible git platform built in Rust with a WebAssembly frontend. Unlike GitHub/GitLab clones, Nimbus embraces a single-owner model with powerful plugin architecture for experimentation with CI/CD, code review, and AI collaboration.

## 🚀 Why Nimbus?

- **Not another GitHub clone** - Single-owner instances that federate, not monolithic multi-tenant systems
- **Cloud-native from day one** - Built for Kubernetes, not retrofitted
- **Plugin everything** - Core does git, plugins do everything else
- **Rust + WASM** - Fast backend, interactive frontend, one language
- **AI-ready** - MCP integration for any AI model to participate in code review

## 🎯 Core Philosophy

### Minimal Core, Maximum Extensibility
The core provides rock-solid git operations. Everything else—CI/CD, code review, AI analysis—is a plugin. Bring your own tools.

### Single Owner, Multiple Collaborators
- `code.navicore.tech` - Ed's repos
- `code.friend.dev` - Friend's repos
- No namespaces, no forks, no social features
- Want to fork? Clone to YOUR instance

## 🏗️ Architecture

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐
│   Warp      │────▶│  Event Bus   │◀────│   Plugins    │
│  REST API   │     │              │     │              │
└─────────────┘     └──────────────┘     ├─ CI Runners  │
                           ▲              ├─ Reviewers   │
┌─────────────┐           │              └─ AI Models   │
│   Leptos    │           │              
│  WASM UI    │     ┌──────────────┐     
└─────────────┘     │  Git Core    │     
                    │  (git2/gix)   │     
                    └──────────────┘     
```

## 🚦 Status

**Pre-Alpha** - Architecture and design phase

### Roadmap

- [x] Architecture design
- [x] Single-owner model
- [x] Plugin architecture
- [x] API design
- [ ] Core implementation (in progress)
- [ ] Event bus
- [ ] Basic UI
- [ ] First plugin (GitHub Actions runner)
- [ ] MCP integration

## 🛠️ Tech Stack

- **Backend**: Rust, Warp, git2/gitoxide
- **Frontend**: Leptos (Rust → WASM)
- **Database**: PostgreSQL (metadata), Redis (sessions)
- **Storage**: S3-compatible (git objects)
- **Deploy**: Kubernetes native

## 📦 Installation

*Coming soon - Nimbus is in early development*

```bash
# Future installation will be:
helm install nimbus nimbus/nimbus \
  --set owner.email=you@example.com \
  --set domain=code.yourdomain.com
```

## 🤝 Contributing

Nimbus is being built in the open. The primary use case is personal git hosting with collaboration, but the plugin architecture enables community experimentation.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/navicore/nimbus-git
cd nimbus-git

# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

## 🔌 Plugin Examples

### CI Runner Plugin
```rust
impl EventHandler for GitHubActionsRunner {
    async fn handle(&self, event: EventEnvelope) -> Result<()> {
        if let Event::Push { repository, branch, .. } = event.event {
            // Trigger GitHub Actions workflow
            self.trigger_workflow(&repository, &branch).await?;
        }
        Ok(())
    }
}
```

### AI Reviewer Plugin (via MCP)
```rust
impl EventHandler for ClaudeMCP {
    async fn handle(&self, event: EventEnvelope) -> Result<()> {
        if let Event::PullRequestOpened { id, .. } = event.event {
            // Get diff and send to Claude via MCP
            let analysis = self.analyze_pr(id).await?;
            // Post suggestions as PR comments
            self.post_suggestions(id, analysis).await?;
        }
        Ok(())
    }
}
```

## 📝 License

MIT OR Apache-2.0 (your choice)

## 🙏 Acknowledgments

- Inspired by the simplicity of Fossil SCM
- Federation ideas from Forgejo/Gitea
- Plugin architecture influenced by Kubernetes operators

---

**Nimbus** - Because your code deserves better than another GitHub clone.