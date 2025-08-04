---
layout: page
title: "Examples"
permalink: /examples/
---

# 🎯 SigmOS Examples

Explore real-world examples of SigmOS in action. Each example demonstrates different aspects of the language and showcases how to build intelligent, reactive systems.

## 🤖 Basic Examples

### AI Agent
A simple AI agent with personality and conversation capabilities.

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
    on_message: trigger {
      when: input.message != null
      action: respond
    }

  actions:
    respond: prompt {
      system: "You are {{name}} with a {{tone}} personality."
      user: "{{input.message}}"
      model: "gpt-4"
    }
}
```

[View Full Example →](https://github.com/copyleftdev/sigmos/blob/main/examples/agent.sigmos)

### Content Pipeline
An AI-powered content generation and validation pipeline.

```sigmos
spec "ContentPipeline" v1.0 {
  description: "AI-powered content generation with validation"
  
  inputs:
    topic: string
    target_audience: enum("technical", "general", "academic")
    content_type: enum("blog", "documentation", "tutorial")
  
  computed:
    content_prompt: -> "Write a {{content_type}} about {{topic}} for {{target_audience}} audience"
  
  actions:
    generate: prompt {
      system: "You are an expert content creator."
      user: "{{content_prompt}}"
      model: "gpt-4"
    }
    
    validate: prompt {
      system: "Review this content for quality and accuracy."
      user: "Content: {{actions.generate.result}}"
      model: "gpt-4"
    }
}
```

[View Full Example →](https://github.com/copyleftdev/sigmos/blob/main/examples/ai-content-pipeline.sigmos)

## 🏭 Industry Examples

<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem; margin: 2rem 0;">
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>🏥 Healthcare</h3>
    <p>Patient data processing, medical record analysis, and treatment recommendation systems.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/healthcare">View Examples →</a>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>💰 FinTech</h3>
    <p>Risk assessment, fraud detection, and automated trading systems with AI-powered decision making.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/fintech">View Examples →</a>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>🛒 E-Commerce</h3>
    <p>Product recommendations, inventory management, and customer service automation.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/ecommerce">View Examples →</a>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>🏭 Manufacturing</h3>
    <p>Quality control, predictive maintenance, and supply chain optimization systems.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/manufacturing">View Examples →</a>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>🚚 Logistics</h3>
    <p>Route optimization, demand forecasting, and automated dispatch systems.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/logistics">View Examples →</a>
  </div>
  
  <div style="padding: 1.5rem; border: 1px solid #e1e5e9; border-radius: 0.5rem;">
    <h3>🏙️ Smart City</h3>
    <p>Traffic management, energy optimization, and citizen service automation.</p>
    <a href="https://github.com/copyleftdev/sigmos/tree/main/examples/smart-city">View Examples →</a>
  </div>
</div>

## 🔧 Advanced Patterns

### User Management System
Complete user lifecycle management with validation and security.

```sigmos
spec "UserManagement" v1.0 {
  description: "Complete user lifecycle management system"
  
  types:
    User: struct {
      id: string
      email: string
      role: enum("admin", "user", "guest")
      created_at: string
      permissions: list<string>
    }
  
  inputs:
    action: enum("create", "update", "delete", "authenticate")
    user_data: User { optional: true }
    credentials: struct {
      email: string
      password: string { secret: true }
    } { optional: true }
  
  constraints:
    valid_email: user_data.email ~= /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    strong_password: len(credentials.password) >= 8
  
  events:
    on_create: trigger {
      when: input.action == "create"
      action: create_user
    }
    
    on_auth: trigger {
      when: input.action == "authenticate"
      action: authenticate_user
    }
  
  actions:
    create_user: block {
      validate_input: -> constraints.valid_email && user_data.role != null
      hash_password: -> hash(credentials.password)
      store_user: rest.post("/api/users", user_data)
    }
    
    authenticate_user: block {
      verify_credentials: rest.post("/api/auth", credentials)
      generate_token: -> jwt.sign(verify_credentials.user_id)
    }
}
```

[View Full Example →](https://github.com/copyleftdev/sigmos/blob/main/examples/user-management.sigmos)

## 🎓 Learning Path

1. **Start Simple**: Begin with the basic [Agent example](https://github.com/copyleftdev/sigmos/blob/main/examples/agent.sigmos)
2. **Add Complexity**: Explore the [Content Pipeline](https://github.com/copyleftdev/sigmos/blob/main/examples/ai-content-pipeline.sigmos)
3. **Industry Focus**: Choose an industry example that matches your domain
4. **Advanced Patterns**: Study the [User Management](https://github.com/copyleftdev/sigmos/blob/main/examples/user-management.sigmos) system

## 🛠️ Running Examples

```bash
# Clone the repository
git clone https://github.com/copyleftdev/sigmos.git
cd sigmos

# Run any example
sigmos run examples/agent.sigmos

# Run with custom inputs
sigmos run examples/agent.sigmos --input name="Alice" --input tone="friendly"

# Validate an example
sigmos validate examples/user-management.sigmos
```

## 📚 Next Steps

- [User Guide](/sigmos/docs/user-guide) - Learn the language fundamentals
- [Developer Guide](/sigmos/docs/developer-guide) - Build and extend SigmOS
- [API Reference](/sigmos/docs/api-reference) - Complete language reference

---

<div style="text-align: center; padding: 2rem; background: #f8f9fa; border-radius: 0.5rem;">
  <h3>Have an example to share?</h3>
  <p>We'd love to see what you're building with SigmOS!</p>
  <a href="https://github.com/copyleftdev/sigmos/issues/new" style="background: #667eea; color: white; padding: 0.8rem 1.5rem; text-decoration: none; border-radius: 0.5rem;">Submit Your Example</a>
</div>
