# SIGMOS: Sigma Modular Operating Spec — Final Language Specification

## 1. Overview

**SIGMOS** is a next-generation Domain-Specific Language (DSL) designed to define, orchestrate, and execute AI-native, composable, reactive, and multimodal systems. It combines typed schema validation, prompt-based AI workflows, plugin extensibility, and runtime orchestration into a unified language tailored for the modern era of intelligent systems.

## 2. Core Principles

* **Declarative-first**: Express what should happen.
* **Typed & Validated**: Strong types, constraint logic, schema compliance.
* **AI-Native**: Prompts, LLM inference, dynamic generation are first-class citizens.
* **Composable**: Modular specs, reusable patterns, namespaced imports.
* **Extensible**: Plugin-based architecture with secure runtime extensions.
* **Reactive**: Trigger-based, event-driven, lifecycle-aware.
* **Secure**: Field permissioning, trusted imports, deterministic evaluation.

## 3. Grammar and Syntax

SIGMOS uses a PEG grammar (via `pest`) and supports whitespace-insensitive block-based syntax.

### 3.1 Core Keywords

* `spec`, `inputs`, `computed`, `events`, `constraints`, `lifecycle`, `types`, `extensions`, `actions`, `block`

### 3.2 Type System

```
string, int, float, bool, null
list<T>, map<K,V>, enum(...), union(...), struct {...}, ref(...), prompt, text.generate
```

### 3.3 Modifiers

* `optional`, `readonly`, `default`, `computed`, `secret`, `generate`, `ref`, `@doc`

## 4. Example File

```sigmos
spec "Agent" v1.0 {
  description: "Defines an AI Agent with LLM prompt capabilities."

  inputs:
    name: string
    tone: enum("friendly", "hostile")
    api_key: string { secret: true }

  computed:
    greeting: -> "Hello, I'm {{name}}, and I'm {{tone}}."

  events:
    on_create(agent): mcp.call("mission.begin", {
      auth: ref("api_key"),
      payload: { id: agent.name }
    })

  constraints:
    assert name != ""
    ensure tone in ["friendly", "hostile"]

  lifecycle:
    before: validate
    after: log("agent init complete")
}
```

## 5. Type System & Validation

* First-class generics and inline constraints
* Custom types via `types {}` block and external plugins
* Deterministic evaluation and scoped dependency resolution

## 6. Extensions & Plugin Runtime

```sigmos
extensions {
  mcp: import("sigmos.std.net.mcp@1.0")
  qdrant: import("sigmos.ai.qdrant.embed@0.3")
}
```

* Implemented in Rust using the `Extension` trait
* Resolved at runtime and namespaced
* Fully sandboxed, signature-verifiable

## 7. Event Triggers

```sigmos
events {
  on_change(input): regenerate()
  on_error(task): mcp.call("mission.retry", { id: task.id })
}
```

* Supports `on_create`, `on_change`, `on_error`, custom signal events

## 8. Lifecycle Model

```sigmos
lifecycle {
  before: validate
  after: persist()
  finally: cleanup()
}
```

* Defines run-phase boundaries and hooks into runtime orchestration

## 9. CLI Tooling

```bash
sigmos validate <file.spec∞>
sigmos transpile <file> --to json
sigmos run <file>
sigmos install <plugin>
```

* Built using `clap`, `serde`, and `miette`
* Full `cargo` workspace structure for subcommands

## 10. Versioning & Imports

```sigmos
spec uses [
  "sigmos.std.prompts@2.1",
  "sigmos.agent.auth@1.0"
]
```

* Semantic version resolution
* Forward-compatible imports and override modules

## 11. Security Model

* Field secrets (e.g. `secret: true`)
* Signature verification of imported modules
* Registry integrity hashes
* Runtime sandboxing policies

## 12. AI-Native Constructs

```sigmos
summary: text.generate("Summarize this profile: {{input}}")
prompt: prompt.embed("You are a friendly AI agent...")
```

* Supports: zero-shot, few-shot, templated prompt generation
* Output constraints (e.g. `maxLength`, `type` validation)

## 13. Introspection & AI Reflection

```sigmos
@doc("Initializes a mission")
action start_mission {
  type: mcp.call
  topic: "mission.begin"
  payload: { id: task.id }
}
```

* Enables self-documenting specs and LLM-driven docgen
* CLI support for `sigmos describe` or `sigmos explain`

## 14. Developer Tooling

* Plugin scaffold: `sigmos plugin new <name>`
* `insta` and `trybuild` testing
* GitHub Actions CI for parser/linter/test workflows

## 15. Ecosystem Layout

```bash
sigmos/
├── crates/
│   ├── core/      # Grammar, AST, parser
│   ├── runtime/   # Evaluation engine
│   ├── cli/       # CLI binary
│   ├── plugins/   # Official plugins (mcp, rest, etc.)
│   ├── transpiler/ # Export formats (json, yaml, etc.)
├── docs/
├── examples/
├── spec/          # DSL EBNF, version log
├── registry/      # Plugin manifest JSON
├── tests/
```

## 16. Community & Governance

* GitHub Org: `sigmos`
* MIT / Apache dual license
* Docusaurus docs + playground (planned)
* Plugin registry with signature metadata
* Contributor onboarding: `good-first-issue`, plugin guides

---

SIGMOS is the modular operating specification for orchestrating cognition, automation, and intelligence across AI-native systems. This language isn't just a config format — it's a future-proof DSL designed by Lead Sigma for the builders of what comes next.
