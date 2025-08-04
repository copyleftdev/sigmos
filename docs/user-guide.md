# SIGMOS User Guide

Welcome to SIGMOS (Sigma Modular Operating Spec) - a next-generation DSL for AI-native, composable, reactive systems.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Basic Concepts](#basic-concepts)
4. [Writing Your First Spec](#writing-your-first-spec)
5. [Expressions and Variables](#expressions-and-variables)
6. [Built-in Functions](#built-in-functions)
7. [Plugins](#plugins)
8. [Advanced Features](#advanced-features)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)

## Introduction

SIGMOS is designed to be:
- **Declarative-first**: Describe what you want, not how to achieve it
- **Typed & validated**: Strong type system with compile-time validation
- **AI-native**: Built-in support for prompts and LLM inference
- **Composable**: Modular specifications that can be combined
- **Extensible**: Plugin architecture for custom functionality
- **Reactive**: Event-driven system with lifecycle management
- **Secure**: Field-level permissions and access control

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/copyleftdev/sigmos
cd sigmos

# Build the project
cargo build --release

# Run the CLI
./target/release/sigmos --help
```

### Your First Command

```bash
# Parse and validate a SIGMOS specification
sigmos parse examples/hello-world.sigmos

# Execute a specification
sigmos run examples/hello-world.sigmos

# Transpile to JSON
sigmos transpile examples/hello-world.sigmos --format json
```

## Basic Concepts

### Specifications

A SIGMOS specification is a declarative description of a system or workflow. Every spec has:

- **Name**: A unique identifier for the specification
- **Version**: Semantic version (major.minor.patch)
- **Description**: Human-readable description (optional)
- **Fields**: Input, computed, and output fields
- **Events**: Reactive event handlers
- **Constraints**: Validation rules
- **Lifecycle**: Before/after execution hooks

### Basic Syntax

```sigmos
spec "MyApplication" v1.0.0 {
    description: "A simple SIGMOS application"
    
    input name: String {
        description: "User's name"
        required: true
    }
    
    computed greeting: String = "Hello, ${name}!"
    
    event onCreate(name) {
        log("User ${name} created")
    }
}
```

## Writing Your First Spec

Let's create a simple greeting application:

```sigmos
spec "GreetingApp" v1.0.0 {
    description: "A friendly greeting application"
    
    // Input field - user provides their name
    input name: String {
        description: "Your name"
        required: true
        default: "World"
    }
    
    // Computed field - automatically calculated
    computed greeting: String = "Hello, ${name}! Welcome to SIGMOS."
    
    // Another computed field using functions
    computed name_length: Number = len(name)
    
    // Conditional computed field
    computed formal_greeting: String = name_length > 10 
        ? "Good day, ${name}. It's a pleasure to meet you."
        : "Hi ${name}!"
    
    // Event handler
    event onCreate(name) {
        log("New user: ${name}")
    }
    
    // Constraint
    constraint {
        assert: name_length > 0
        message: "Name cannot be empty"
    }
}
```

### Running Your Spec

```bash
# Save the above as greeting.sigmos
sigmos run greeting.sigmos --input name="Alice"
```

## Expressions and Variables

SIGMOS supports rich expressions for computed fields and conditions.

### Basic Types

```sigmos
// String literals
computed message: String = "Hello World"

// Numbers
computed count: Number = 42
computed pi: Number = 3.14159

// Booleans
computed is_active: Boolean = true
computed is_ready: Boolean = false
```

### Variables and Identifiers

```sigmos
input user_name: String
computed display_name: String = user_name
```

### Arithmetic Operations

```sigmos
input a: Number = 10
input b: Number = 5

computed sum: Number = a + b           // 15
computed difference: Number = a - b    // 5
computed product: Number = a * b       // 50
computed quotient: Number = a / b      // 2
computed remainder: Number = a % b     // 0
```

### String Operations

```sigmos
input first_name: String = "John"
input last_name: String = "Doe"

// String concatenation
computed full_name: String = first_name + " " + last_name

// String interpolation
computed greeting: String = "Hello, ${first_name} ${last_name}!"
```

### Comparison Operations

```sigmos
input age: Number = 25

computed is_adult: Boolean = age >= 18
computed is_senior: Boolean = age >= 65
computed age_group: String = age < 18 ? "minor" : "adult"
```

### Logical Operations

```sigmos
input is_member: Boolean = true
input has_discount: Boolean = false

computed can_purchase: Boolean = is_member && !has_discount
computed needs_verification: Boolean = !is_member || age < 21
```

### Conditional Expressions

```sigmos
input score: Number = 85

computed grade: String = score >= 90 ? "A" :
                        score >= 80 ? "B" :
                        score >= 70 ? "C" :
                        score >= 60 ? "D" : "F"
```

## Built-in Functions

SIGMOS provides several built-in functions for common operations:

### String Functions

```sigmos
input text: String = "  Hello World  "

computed text_length: Number = len(text)        // 15
computed uppercase: String = upper(text)        // "  HELLO WORLD  "
computed lowercase: String = lower(text)        // "  hello world  "
computed trimmed: String = trim(text)           // "Hello World"
```

### Math Functions

```sigmos
input value: Number = -42.7

computed absolute: Number = abs(value)          // 42.7
```

### Array and Object Functions

```sigmos
input items: Array = ["apple", "banana", "cherry"]
input user: Object = {"name": "Alice", "age": 30}

computed item_count: Number = len(items)        // 3
computed field_count: Number = len(user)        // 2
computed first_item: String = items[0]          // "apple"
computed user_name: String = user.name          // "Alice"
```

## Plugins

SIGMOS supports plugins for extending functionality. Two official plugins are available:

### MCP Plugin (AI Integration)

```sigmos
spec "AIAssistant" v1.0.0 {
    input prompt: String
    
    computed ai_response: String = mcp.complete(prompt, {
        model: "gpt-4",
        max_tokens: 100
    })
    
    computed summary: String = mcp.analyze(ai_response, {
        task: "summarize"
    })
}
```

### REST Plugin (HTTP API)

```sigmos
spec "WeatherApp" v1.0.0 {
    input city: String = "San Francisco"
    
    computed weather_data: Object = rest.get("/weather", {
        q: city,
        appid: "your-api-key"
    })
    
    computed temperature: Number = weather_data.main.temp
    computed description: String = weather_data.weather[0].description
}
```

## Advanced Features

### String Templates

```sigmos
input user: Object = {"name": "Alice", "role": "admin"}

computed welcome_message: String = `
Welcome, ${user.name}!
Your role: ${user.role}
Today is a great day to use SIGMOS.
`
```

### Complex Expressions

```sigmos
input users: Array = [
    {"name": "Alice", "age": 30},
    {"name": "Bob", "age": 25}
]

computed adult_users: Array = users.filter(user => user.age >= 18)
computed average_age: Number = users.reduce((sum, user) => sum + user.age, 0) / len(users)
```

### Event Handling

```sigmos
spec "UserManager" v1.0.0 {
    input user_data: Object
    
    event onCreate(user_data) {
        // Log user creation
        log("User created: ${user_data.name}")
        
        // Send welcome email
        rest.post("/send-email", {
            to: user_data.email,
            subject: "Welcome!",
            body: "Welcome to our platform, ${user_data.name}!"
        })
    }
    
    event onUpdate(user_data) {
        log("User updated: ${user_data.name}")
    }
    
    event onError(error) {
        log("Error occurred: ${error.message}")
    }
}
```

### Lifecycle Hooks

```sigmos
spec "DataProcessor" v1.0.0 {
    input data: Array
    
    lifecycle before {
        // Validate data before processing
        validate_data(data)
    }
    
    computed processed_data: Array = transform(data)
    
    lifecycle after {
        // Clean up resources
        cleanup()
    }
}
```

## Best Practices

### 1. Use Descriptive Names

```sigmos
// Good
computed user_full_name: String = first_name + " " + last_name
computed is_eligible_for_discount: Boolean = age >= 65

// Avoid
computed x: String = a + " " + b
computed flag: Boolean = n >= 65
```

### 2. Add Descriptions

```sigmos
input email: String {
    description: "User's email address for notifications"
    required: true
    pattern: "^[^@]+@[^@]+\.[^@]+$"
}
```

### 3. Use Constraints for Validation

```sigmos
input age: Number {
    description: "User's age in years"
    required: true
}

constraint {
    assert: age >= 0 && age <= 150
    message: "Age must be between 0 and 150"
}
```

### 4. Organize Complex Logic

```sigmos
// Break complex expressions into smaller parts
computed base_price: Number = item_price * quantity
computed discount_amount: Number = base_price * discount_rate
computed tax_amount: Number = (base_price - discount_amount) * tax_rate
computed total_price: Number = base_price - discount_amount + tax_amount
```

### 5. Handle Errors Gracefully

```sigmos
computed safe_division: Number = denominator != 0 
    ? numerator / denominator 
    : 0

event onError(error) {
    log("Error: ${error.message}")
    // Implement fallback behavior
}
```

## Troubleshooting

### Common Issues

#### 1. Parse Errors

```
Error: Unexpected token at line 5, column 12
```

**Solution**: Check syntax, ensure proper quotes, brackets, and semicolons.

#### 2. Type Mismatches

```
Error: Cannot assign String to Number field
```

**Solution**: Ensure expressions return the correct type or use type conversion.

#### 3. Undefined Variables

```
Error: Variable 'user_name' is not defined
```

**Solution**: Ensure all variables are declared as inputs or computed fields.

#### 4. Plugin Errors

```
Error: Plugin 'mcp' not found
```

**Solution**: Ensure plugins are properly configured and available.

### Getting Help

- **Documentation**: Check the [API Reference](api-reference.md)
- **Examples**: Browse the `examples/` directory
- **Issues**: Report bugs on GitHub
- **Community**: Join our Discord server

## Next Steps

- Read the [Developer Guide](developer-guide.md) to learn about extending SIGMOS
- Explore the [API Reference](api-reference.md) for detailed function documentation
- Check out [Examples](../examples/) for real-world use cases
- Learn about [Plugin Development](plugin-development.md)

---

*Happy coding with SIGMOS! 🚀*
