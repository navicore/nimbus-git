# MCP (Model Context Protocol) Integration

## Overview

MCP enables AI assistants to interact with Nimbus through structured tools and resources. This allows for intelligent code review, automated refactoring, and AI-powered development workflows.

## Architecture

```
┌─────────────┐     MCP Protocol      ┌──────────────┐
│ AI Assistant│◄──────────────────────►│ MCP Server   │
│   (Claude)  │                        │  (Nimbus)    │
└─────────────┘                        └──────┬───────┘
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │   Event Bus     │
                                    └─────────────────┘
```

## MCP Server Design

The MCP server will be a separate binary that:
1. Connects to Nimbus via WebSocket for real-time events
2. Exposes MCP tools for AI interaction
3. Manages conversation context and state

## Exposed Tools

### Repository Management
```typescript
// List repositories
tool list_repos() -> Repository[]

// Get repository details
tool get_repo(name: string) -> Repository

// Clone repository locally for AI
tool clone_repo(name: string) -> LocalPath
```

### Code Operations
```typescript
// Read file from repository
tool read_file(
  repo: string,
  path: string,
  ref?: string // branch/tag/commit
) -> FileContent

// Write file to repository
tool write_file(
  repo: string,
  path: string,
  content: string,
  message: string
) -> Commit

// Search code
tool search_code(
  query: string,
  repo?: string,
  language?: string
) -> SearchResult[]
```

### Git Operations
```typescript
// Create branch
tool create_branch(
  repo: string,
  name: string,
  from?: string
) -> Branch

// Create pull request
tool create_pr(
  repo: string,
  from_branch: string,
  to_branch: string,
  title: string,
  description: string
) -> PullRequest

// Review pull request
tool review_pr(
  repo: string,
  pr_id: number,
  comments: Comment[],
  approval?: boolean
) -> Review
```

### CI/CD Integration
```typescript
// Trigger CI run
tool trigger_ci(
  repo: string,
  branch: string,
  workflow?: string
) -> CiRun

// Get CI status
tool get_ci_status(
  repo: string,
  run_id: string
) -> CiStatus

// Get test results
tool get_test_results(
  repo: string,
  run_id: string
) -> TestResults
```

### AI-Specific Tools
```typescript
// Analyze code for issues
tool analyze_code(
  repo: string,
  path?: string,
  checks?: string[] // ["security", "performance", "style"]
) -> Analysis[]

// Generate code
tool generate_code(
  repo: string,
  prompt: string,
  context_files?: string[]
) -> GeneratedCode

// Refactor code
tool refactor_code(
  repo: string,
  path: string,
  refactor_type: string, // "extract_function", "rename", etc.
  params: RefactorParams
) -> Diff
```

## Event Subscriptions

The MCP server subscribes to events via the Event Bus:

```rust
// Events AI cares about
enum AiRelevantEvent {
    // Code review requests
    ReviewRequested { pr: PullRequest },
    
    // CI failures that need investigation
    CiRunFailed { run: CiRun, logs: String },
    
    // Security alerts
    SecurityIssueDetected { issue: SecurityIssue },
    
    // Direct AI requests
    AiAnalysisRequested { target: AnalysisTarget },
}
```

## Implementation Plan

### Phase 1: Core MCP Server
```rust
// crates/nimbus-mcp/src/main.rs
use mcp_rust::{Server, Tool, Resource};
use nimbus_events::{EventBus, EventHandler};

struct NimbusMcpServer {
    nimbus_client: NimbusClient,
    event_bus: Arc<EventBus>,
}

impl McpServer for NimbusMcpServer {
    async fn handle_tool_call(&self, tool: &str, params: Value) -> Result<Value> {
        match tool {
            "list_repos" => self.list_repos().await,
            "read_file" => self.read_file(params).await,
            // ... other tools
        }
    }
}
```

### Phase 2: Event Handler
```rust
// Subscribe to events for proactive AI assistance
struct AiEventHandler {
    mcp_server: Arc<NimbusMcpServer>,
}

#[async_trait]
impl EventHandler for AiEventHandler {
    async fn handle(&self, event: EventEnvelope) -> Result<()> {
        match event.event {
            Event::ReviewRequested { .. } => {
                // Trigger automatic code review
                self.trigger_code_review(event).await?;
            }
            Event::CiRunFailed { .. } => {
                // Analyze failure and suggest fixes
                self.analyze_ci_failure(event).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

### Phase 3: Context Management
```rust
// Maintain conversation context
struct ConversationContext {
    current_repo: Option<String>,
    current_branch: Option<String>,
    open_files: Vec<String>,
    recent_commits: Vec<Commit>,
}

impl NimbusMcpServer {
    // Provide relevant context to AI
    async fn get_context(&self) -> ConversationContext {
        // Gather context from current state
    }
    
    // Update context based on AI actions
    async fn update_context(&self, action: &str, params: &Value) {
        // Track what the AI is working on
    }
}
```

## Security Considerations

1. **Authentication**: MCP server authenticates with Nimbus using API tokens
2. **Rate Limiting**: Prevent AI from overwhelming the system
3. **Sandboxing**: AI operations run in isolated environments
4. **Audit Logging**: Track all AI actions for review
5. **Permissions**: Granular control over what AI can access/modify

## Configuration

```toml
# nimbus-mcp.toml
[server]
port = 5000
nimbus_url = "http://localhost:3000"
nimbus_token = "${NIMBUS_API_TOKEN}"

[tools]
enabled = ["read", "write", "analyze", "review"]
disabled = ["delete", "force_push"]

[limits]
max_file_size = "10MB"
max_files_per_request = 100
rate_limit = "100/minute"

[ai]
model = "claude-3-opus"
temperature = 0.3
max_tokens = 4000
```

## Usage Examples

### Example 1: Automated Code Review
```typescript
// AI receives ReviewRequested event
const pr = await tool.get_pr("nimbus-git", 42);
const diff = await tool.get_diff("nimbus-git", pr.from_branch, pr.to_branch);

// Analyze the changes
const issues = await tool.analyze_code("nimbus-git", {
  checks: ["security", "performance", "best_practices"]
});

// Post review
await tool.review_pr("nimbus-git", 42, {
  comments: issues.map(i => ({
    path: i.file,
    line: i.line,
    body: i.suggestion
  })),
  approval: issues.filter(i => i.severity === "high").length === 0
});
```

### Example 2: CI Failure Investigation
```typescript
// AI receives CiRunFailed event
const logs = await tool.get_ci_logs("nimbus-git", run_id);
const test_results = await tool.get_test_results("nimbus-git", run_id);

// Analyze failure
const analysis = await tool.analyze_failure(logs, test_results);

// Create fix branch
await tool.create_branch("nimbus-git", "fix-ci-failure", "main");

// Apply fixes
for (const fix of analysis.fixes) {
  await tool.write_file("nimbus-git", fix.path, fix.content, 
    `Fix: ${fix.description}`);
}

// Create PR with explanation
await tool.create_pr("nimbus-git", "fix-ci-failure", "main", 
  "Fix CI failures", analysis.explanation);
```

## Benefits

1. **Intelligent Assistance**: AI can help with code reviews, debugging, and refactoring
2. **Automation**: Routine tasks handled automatically
3. **Learning**: AI learns from codebase patterns and conventions
4. **Collaboration**: AI acts as a team member, not just a tool
5. **Extensibility**: Easy to add new AI capabilities via MCP tools

## Next Steps

1. Define MCP tool schemas in detail
2. Implement basic MCP server in Rust
3. Create event handlers for AI triggers
4. Build context management system
5. Add security and rate limiting
6. Create example AI workflows
7. Document best practices