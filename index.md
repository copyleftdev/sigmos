---
layout: home
title: "SigmOS: Sigma Modular Operating Spec"
---

<div class="hero-section" style="text-align: center; padding: 2rem 0; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; margin: -2rem -2rem 2rem -2rem; border-radius: 0 0 1rem 1rem;">
  <h1 style="font-size: 3rem; margin-bottom: 1rem; text-shadow: 2px 2px 4px rgba(0,0,0,0.3);">🚀 SigmOS</h1>
  <p style="font-size: 1.3rem; margin-bottom: 2rem; opacity: 0.9;">A next-generation Domain-Specific Language for AI-native systems</p>
  <div style="display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap;">
    <a href="/sigmos/docs/user-guide" style="background: rgba(255,255,255,0.2); color: white; padding: 0.8rem 1.5rem; text-decoration: none; border-radius: 0.5rem; backdrop-filter: blur(10px); transition: all 0.3s;">📖 Get Started</a>
    <a href="/sigmos/examples" style="background: rgba(255,255,255,0.2); color: white; padding: 0.8rem 1.5rem; text-decoration: none; border-radius: 0.5rem; backdrop-filter: blur(10px); transition: all 0.3s;">🎯 Examples</a>
    <a href="https://github.com/copyleftdev/sigmos" style="background: rgba(255,255,255,0.2); color: white; padding: 0.8rem 1.5rem; text-decoration: none; border-radius: 0.5rem; backdrop-filter: blur(10px); transition: all 0.3s;">⭐ GitHub</a>
  </div>
</div>

## ✨ What is SigmOS?

SigmOS is a revolutionary Domain-Specific Language (DSL) that brings **declarative-first** thinking to AI-native system orchestration. Think of it as the missing piece between configuration files and full programming languages—designed specifically for the era of intelligent, reactive systems.

```sigmos
spec "AIAgent" v1.0 {
  description: "An intelligent agent with multimodal capabilities"
  
  inputs:
    name: string
    personality: enum("helpful", "creative", "analytical")
    api_key: string { secret: true }
  
  computed:
    greeting: -> "Hello! I'm {{name}}, your {{personality}} AI assistant."
  
  actions:
    respond: prompt {
      system: "You are a {{personality}} AI assistant named {{name}}."
      user: "{{input.message}}"
      model: "gpt-4"
    }
}
```

## 🎯 Key Features

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; margin: 2rem 0;">
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>🧠 AI-Native</h3>
    <p>Prompts, LLM inference, and dynamic generation as first-class citizens. Build intelligent systems that think and adapt.</p>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>🔧 Composable</h3>
    <p>Modular specs, reusable patterns, and namespaced imports. Build complex systems from simple, tested components.</p>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>⚡ Reactive</h3>
    <p>Trigger-based, event-driven, lifecycle-aware systems that respond intelligently to changing conditions.</p>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>🛡️ Secure</h3>
    <p>Field permissioning, trusted imports, and deterministic evaluation keep your systems safe and predictable.</p>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>🎨 Typed & Validated</h3>
    <p>Strong types, constraint logic, and schema compliance ensure your specifications are correct by construction.</p>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem; background: #f8f9fa;">
    <h3>🔌 Extensible</h3>
    <p>Plugin-based architecture with secure runtime extensions. Extend functionality without compromising security.</p>
  </div>
</div>

## 🚀 Quick Start

Get up and running with SigmOS in minutes:

```bash
# Clone the repository
git clone https://github.com/copyleftdev/sigmos.git
cd sigmos

# Build the project
cargo build --release

# Install the CLI
cargo install --path crates/cli

# Run your first spec
sigmos run examples/agent.sigmos
```

## 🏗️ Use Cases

SigmOS excels in scenarios where you need to orchestrate intelligent, reactive systems:

- **AI Workflow Orchestration**: Chain LLM calls, process multimodal data, and handle complex reasoning workflows
- **Intelligent Automation**: Build systems that adapt and learn from their environment
- **API Orchestration**: Compose complex API interactions with built-in error handling and retries
- **Content Generation Pipelines**: Create sophisticated content workflows with AI-powered generation and validation
- **Smart System Configuration**: Define infrastructure that responds intelligently to changing conditions

## 🌟 Why SigmOS?

Traditional configuration languages weren't designed for the AI era. SigmOS bridges the gap between static configuration and full programming languages, offering:

- **Declarative Simplicity**: Express *what* should happen, not *how*
- **AI-First Design**: Native support for prompts, models, and intelligent workflows
- **Type Safety**: Catch errors at specification time, not runtime
- **Composability**: Build complex systems from simple, reusable components
- **Future-Proof**: Designed for the next generation of intelligent systems

---

<div style="text-align: center; padding: 2rem; background: #f8f9fa; border-radius: 0.5rem; margin: 2rem 0;">
  <h2>Ready to build the future?</h2>
  <p style="font-size: 1.1rem; margin-bottom: 1.5rem;">Join the community building the next generation of intelligent systems.</p>
  <a href="/sigmos/docs/user-guide" style="background: #667eea; color: white; padding: 1rem 2rem; text-decoration: none; border-radius: 0.5rem; font-weight: bold; display: inline-block;">Get Started Now →</a>
</div>
