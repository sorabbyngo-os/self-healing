# ai-sandbox scope expansion — design notes

## Current state
ai-sandbox creates isolated namespaces for AI agents only.
Uses seccomp + Linux namespaces (PID, net, mount, IPC).

## What to change

### 1. Make ai-sandbox the system-wide sandbox layer

Rename the core sandbox primitives to `sorabbyngo_sandbox`:

```rust
// Currently: AiSandbox::new(agent_manifest)
// Change to:  Sandbox::new(SandboxPolicy)

pub enum SandboxPolicy {
    Agent(AgentManifest),   // AI agent — policy from manifest
    App(AppBundle),         // Regular app — policy from app bundle
    DevToolbox(Project),    // Dev toolbox — project-scoped
    SecurityTool(Tool),     // Security tool — Kali-style, airgapped
}
```

### 2. Default deny at process spawn

In `execve` LSM hook (add to cap_token_lsm.c):

```c
static int cap_token_bprm_check(struct linux_binprm *bprm)
{
    // Every new exec starts in a minimal namespace
    // capd is notified and must install tokens before
    // the process can do anything useful
    return sorabbyngo_prepare_sandbox(bprm->mm);
}
```

### 3. Namespace factory

ai-sandbox should expose a `SandboxFactory`:

```rust
pub struct SandboxFactory;

impl SandboxFactory {
    pub fn for_agent(manifest: &AgentManifest) -> Sandbox { ... }
    pub fn for_app(bundle: &AppBundle)           -> Sandbox { ... }
    pub fn for_toolbox(project: &str)            -> Sandbox { ... }
    pub fn for_security_tool(tool: &str)         -> Sandbox { ... }
}
```

### 4. Integration points
- capd calls SandboxFactory at agent spawn
- domain-mode calls SandboxFactory::for_security_tool in security mode
- builder-engine uses SandboxFactory::for_app when building app bundles
- byngox uses SandboxFactory::for_agent for each of its 8 agents

## Files to modify in ai-sandbox repo
- src/sandbox.rs   → add SandboxPolicy enum + SandboxFactory
- src/namespace.rs → generalise from AI-only to all process types
- src/seccomp.rs   → add per-policy seccomp filter sets
- src/lib.rs       → re-export SandboxFactory as top-level API
