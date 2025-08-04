# SIGMOS API Reference

Complete reference for SIGMOS language constructs, built-in functions, and plugin APIs.

## Table of Contents

1. [Language Syntax](#language-syntax)
2. [Built-in Functions](#built-in-functions)
3. [Expression Types](#expression-types)
4. [Plugin APIs](#plugin-apis)
5. [Error Types](#error-types)
6. [CLI Commands](#cli-commands)

## Language Syntax

### Specification Declaration

```sigmos
spec "<name>" v<major>.<minor>.<patch> {
    // Specification body
}
```

**Parameters:**
- `name`: String - Unique identifier for the specification
- `major.minor.patch`: Semantic version number

**Example:**
```sigmos
spec "UserManager" v1.2.3 {
    description: "Manages user accounts and authentication"
}
```

### Field Declarations

#### Input Fields
```sigmos
input <name>: <type> [= <default_value>] {
    [description: "<description>"]
    [required: <boolean>]
    [<modifier>: <value>]
}
```

**Modifiers:**
- `required: Boolean` - Whether the field is required
- `default: Expression` - Default value if not provided
- `readonly: Boolean` - Field cannot be modified after creation
- `secret: Boolean` - Field contains sensitive data
- `generate: Boolean` - Value should be auto-generated

**Example:**
```sigmos
input email: String {
    description: "User's email address"
    required: true
    pattern: "^[^@]+@[^@]+\\.[^@]+$"
}

input age: Number = 18 {
    description: "User's age in years"
    min: 0
    max: 150
}
```

#### Computed Fields
```sigmos
computed <name>: <type> = <expression>
```

**Example:**
```sigmos
computed full_name: String = first_name + " " + last_name
computed is_adult: Boolean = age >= 18
computed greeting: String = "Hello, ${full_name}!"
```

### Event Handlers

```sigmos
event <event_type>(<parameter>) {
    <action>
}
```

**Event Types:**
- `onCreate(parameter)` - Triggered when a resource is created
- `onUpdate(parameter)` - Triggered when a resource is updated
- `onDelete(parameter)` - Triggered when a resource is deleted
- `onError(error)` - Triggered when an error occurs
- `custom("<name>", parameter)` - Custom event type

**Example:**
```sigmos
event onCreate(user) {
    log("New user created: ${user.name}")
    rest.post("/webhook", {
        event: "user_created",
        data: user
    })
}

event onError(error) {
    log("Error occurred: ${error.message}")
    mcp.analyze(error.stack, { task: "debug" })
}
```

### Constraints

```sigmos
constraint {
    assert: <boolean_expression>
    [message: "<error_message>"]
}

constraint {
    ensure: <boolean_expression>
    [message: "<error_message>"]
}
```

**Difference:**
- `assert`: Validates input conditions (checked before execution)
- `ensure`: Validates output conditions (checked after execution)

**Example:**
```sigmos
constraint {
    assert: len(password) >= 8
    message: "Password must be at least 8 characters long"
}

constraint {
    ensure: len(processed_data) > 0
    message: "Processing must produce at least one result"
}
```

### Lifecycle Hooks

```sigmos
lifecycle <phase> {
    <action>
}
```

**Phases:**
- `before` - Execute before main processing
- `after` - Execute after main processing

**Example:**
```sigmos
lifecycle before {
    log("Starting data processing")
    validate_input_data()
}

lifecycle after {
    log("Data processing completed")
    cleanup_temp_files()
}
```

## Built-in Functions

### String Functions

#### `len(value: String|Array|Object) -> Number`
Returns the length of a string, array, or object.

```sigmos
computed name_length: Number = len("Alice")        // 5
computed item_count: Number = len([1, 2, 3])       // 3
computed field_count: Number = len({"a": 1, "b": 2}) // 2
```

#### `upper(text: String) -> String`
Converts a string to uppercase.

```sigmos
computed shouting: String = upper("hello world")   // "HELLO WORLD"
```

#### `lower(text: String) -> String`
Converts a string to lowercase.

```sigmos
computed whisper: String = lower("HELLO WORLD")    // "hello world"
```

#### `trim(text: String) -> String`
Removes leading and trailing whitespace.

```sigmos
computed clean: String = trim("  hello world  ")  // "hello world"
```

### Math Functions

#### `abs(value: Number) -> Number`
Returns the absolute value of a number.

```sigmos
computed distance: Number = abs(-42.5)             // 42.5
```

### Type Conversion Functions

#### `string(value: Any) -> String`
Converts any value to its string representation.

```sigmos
computed age_text: String = string(25)             // "25"
computed bool_text: String = string(true)          // "true"
```

#### `number(value: String) -> Number`
Converts a string to a number.

```sigmos
computed parsed_age: Number = number("25")         // 25.0
```

#### `boolean(value: Any) -> Boolean`
Converts any value to a boolean using truthiness rules.

```sigmos
computed is_valid: Boolean = boolean("hello")      // true
computed is_empty: Boolean = boolean("")           // false
```

## Expression Types

### Literals

```sigmos
// String literals
"Hello, World!"
'Single quotes also work'

// Number literals
42
3.14159
-17.5

// Boolean literals
true
false
```

### Variables

```sigmos
// Simple identifier
user_name

// With context
${variable_name}
```

### Arithmetic Operations

```sigmos
a + b          // Addition (also string concatenation)
a - b          // Subtraction
a * b          // Multiplication
a / b          // Division
a % b          // Modulo
```

**Type Rules:**
- Numbers: Standard arithmetic
- Strings: Only `+` (concatenation) is supported
- Mixed types: Error (except string + any for concatenation)

### Comparison Operations

```sigmos
a == b         // Equal
a != b         // Not equal
a < b          // Less than
a <= b         // Less than or equal
a > b          // Greater than
a >= b         // Greater than or equal
```

**Type Rules:**
- Numbers: Numeric comparison
- Strings: Lexicographic comparison
- Booleans: Logical comparison
- Mixed types: Error

### Logical Operations

```sigmos
a && b         // Logical AND (short-circuit)
a || b         // Logical OR (short-circuit)
!a             // Logical NOT
```

**Truthiness Rules:**
- `false`, `null`, `0`, `""`, `[]`, `{}` are falsy
- All other values are truthy

### Conditional Expressions

```sigmos
condition ? if_true : if_false
```

**Example:**
```sigmos
computed status: String = age >= 18 ? "adult" : "minor"
computed grade: String = score >= 90 ? "A" :
                        score >= 80 ? "B" :
                        score >= 70 ? "C" : "F"
```

### Array and Object Access

```sigmos
array[index]           // Array access by index
object[key]            // Object access by key
object.property        // Object property access
```

**Example:**
```sigmos
computed first_item: String = items[0]
computed user_name: String = user.name
computed dynamic_field: Any = data[field_name]
```

### String Templates

```sigmos
"Hello, ${name}! You are ${age} years old."
```

**Features:**
- Variable interpolation: `${variable}`
- Expression interpolation: `${expression}`
- Nested templates supported
- Missing variables show as `${variable_name}`

### Function Calls

```sigmos
// Built-in function
len(text)

// Plugin method
plugin.method(arg1, arg2)

// With named arguments
rest.post("/api/users", {
    name: user_name,
    email: user_email
})
```

## Plugin APIs

### MCP Plugin (AI Integration)

#### Configuration
```rust
McpConfig {
    name: String,
    endpoint: String,
    model: String,
    api_key: Option<String>,
    timeout_seconds: u64,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}
```

#### Methods

##### `complete(prompt: String, options?: Object) -> Object`
Generate text completion using AI model.

```sigmos
computed response: Object = mcp.complete("Explain quantum computing", {
    max_tokens: 100,
    temperature: 0.7
})
computed text: String = response.text
```

##### `embed(text: String, options?: Object) -> Object`
Generate text embeddings.

```sigmos
computed embedding: Object = mcp.embed("Hello world")
computed vector: Array = embedding.vector
```

##### `chat(messages: Array, options?: Object) -> Object`
Multi-turn chat conversation.

```sigmos
computed chat_response: Object = mcp.chat([
    {"role": "user", "content": "What is SIGMOS?"},
    {"role": "assistant", "content": "SIGMOS is a DSL..."},
    {"role": "user", "content": "How do I use it?"}
])
```

##### `analyze(text: String, options: Object) -> Object`
Analyze text for specific tasks.

```sigmos
computed analysis: Object = mcp.analyze(user_feedback, {
    task: "sentiment",
    format: "json"
})
computed sentiment: String = analysis.sentiment
```

### REST Plugin (HTTP API)

#### Configuration
```rust
RestConfig {
    name: String,
    base_url: String,
    default_headers: HashMap<String, String>,
    timeout_seconds: u64,
    max_redirects: u32,
    verify_ssl: bool,
    auth_token: Option<String>,
    user_agent: String,
}
```

#### Methods

##### `get(path: String, params?: Object) -> Object`
HTTP GET request.

```sigmos
computed user_data: Object = rest.get("/users/123")
computed weather: Object = rest.get("/weather", {
    city: "San Francisco",
    units: "metric"
})
```

##### `post(path: String, data: Object, headers?: Object) -> Object`
HTTP POST request.

```sigmos
computed created_user: Object = rest.post("/users", {
    name: "Alice",
    email: "alice@example.com"
})
```

##### `put(path: String, data: Object, headers?: Object) -> Object`
HTTP PUT request.

```sigmos
computed updated_user: Object = rest.put("/users/123", {
    name: "Alice Smith"
})
```

##### `delete(path: String, headers?: Object) -> Object`
HTTP DELETE request.

```sigmos
computed delete_result: Object = rest.delete("/users/123")
```

##### `patch(path: String, data: Object, headers?: Object) -> Object`
HTTP PATCH request.

```sigmos
computed patched_user: Object = rest.patch("/users/123", {
    email: "newemail@example.com"
})
```

## Error Types

### Parse Errors
- **Syntax Error**: Invalid SIGMOS syntax
- **Type Error**: Type mismatch in expressions
- **Reference Error**: Undefined variable or function

### Runtime Errors
- **Evaluation Error**: Error during expression evaluation
- **Plugin Error**: Plugin execution failure
- **Constraint Error**: Constraint validation failure

### Plugin Errors
- **Configuration Error**: Invalid plugin configuration
- **Connection Error**: Network or service connection failure
- **Authentication Error**: Invalid credentials or permissions

## CLI Commands

### `sigmos parse <file>`
Parse and validate a SIGMOS specification.

**Options:**
- `--verbose, -v`: Show detailed parsing information
- `--format <format>`: Output format (text, json)

**Example:**
```bash
sigmos parse examples/user-manager.sigmos --verbose
```

### `sigmos run <file>`
Execute a SIGMOS specification.

**Options:**
- `--input <key=value>`: Provide input values
- `--config <file>`: Plugin configuration file
- `--dry-run`: Validate without executing

**Example:**
```bash
sigmos run user-manager.sigmos --input name="Alice" --input age=30
```

### `sigmos transpile <file>`
Convert SIGMOS to other formats.

**Options:**
- `--format <format>`: Output format (json, yaml, toml)
- `--output <file>`: Output file path
- `--pretty`: Pretty-print output

**Example:**
```bash
sigmos transpile user-manager.sigmos --format json --output user-manager.json --pretty
```

### `sigmos validate <file>`
Validate a SIGMOS specification without execution.

**Options:**
- `--strict`: Enable strict validation mode
- `--config <file>`: Plugin configuration file

**Example:**
```bash
sigmos validate user-manager.sigmos --strict
```

### `sigmos docs`
Generate documentation for SIGMOS specifications.

**Options:**
- `--format <format>`: Documentation format (markdown, html)
- `--output <dir>`: Output directory

**Example:**
```bash
sigmos docs --format html --output ./docs
```

## Type System

### Primitive Types
- `String`: Text data
- `Number`: Numeric data (64-bit floating point)
- `Boolean`: True/false values
- `Null`: Null/undefined value

### Collection Types
- `Array`: Ordered list of values
- `Object`: Key-value pairs (like JSON objects)

### Special Types
- `Any`: Any type (use sparingly)
- `Expression`: Unevaluated expression
- `Function`: Function reference

### Type Annotations
```sigmos
input name: String
computed age: Number
computed is_valid: Boolean
computed items: Array
computed metadata: Object
```

### Type Constraints
```sigmos
input email: String {
    pattern: "^[^@]+@[^@]+\\.[^@]+$"
}

input age: Number {
    min: 0
    max: 150
}

input tags: Array {
    min_length: 1
    max_length: 10
}
```

---

This API reference provides comprehensive documentation for all SIGMOS language features, built-in functions, and plugin APIs. For more examples and tutorials, see the [User Guide](user-guide.md) and [Developer Guide](developer-guide.md).

*Reference current as of SIGMOS v0.1.0*
