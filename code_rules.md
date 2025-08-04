| Principle                              | Expectation                                                                      |
| -------------------------------------- | -------------------------------------------------------------------------------- |
| **Idiomatic Rust**                     | Zero unsafe, strict ownership model, Clippy clean                                |
| **Highly Modular**                     | Each component (parser, AST, runtime, plugins) is decoupled and trait-oriented   |
| **Extensive Docstrings**               | Every public struct, enum, and function includes testable `///` and `# Examples` |
| **Doctest Coverage**                   | Doc examples run in CI (no rot allowed)                                          |
| **Property-based Testing**             | `proptest` for fuzz-style input validation                                       |
| **Error Clarity**                      | `miette` or `thiserror` with rich diagnostic context                             |
| **Strong Typing Over Strings**         | Avoid `Stringly` code, favor `newtype` patterns and tagged enums                 |
| **Spec-Focused Contracts**             | Spec-level assertions baked into runtime via validation traits                   |
| **Clippy / Fmt Enforced**              | CI enforces formatting and lint compliance (no warnings, no excuses)             |
| **Zero Global State**                  | Runtime and registries are explicit, owned, injectable                           |
| **Functional Core / Imperative Shell** | Evaluation is pure logic, I/O is opt-in, injectable, and tested separately       |
