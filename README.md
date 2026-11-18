# AutoMorphic

> **Zero-boilerplate bidirectional struct mapping in Rust.**
> Generate `impl Morph<T>` automatically with `#[derive(AutoMorph)]` — zero runtime cost.

[![Crates.io](https://img.shields.io/crates/v/auto_morphic.svg)](https://crates.io/crates/auto_morphic)
[![Documentation](https://docs.rs/auto_morphic/badge.svg)](https://docs.rs/auto_morphic)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

---

## Vision

In Rust, converting between similar structs — Entity ↔ DTO, Request ↔ Response, DatabaseModel ↔ ApiModel — requires
writing repetitive and error-prone code. Every time a field changes, you have to manually update all the conversions.

**AutoMorphic eliminates this boilerplate entirely.** With a single `#[derive(AutoMorph)]` annotation, the macro
generates at compile time code identical to what you would write by hand: no overhead, no runtime dependencies, no
runtime magic. Just pure and efficient Rust.

### Project Principles

1. **Zero cost** — The generated code is indistinguishable from hand-written code. Benchmarks prove it.
2. **Bidirectional by default** — `morph_to()` and `morph_from()` always go together.
3. **Explicit and predictable** — Every field maps declaratively; if something is unclear, the compiler tells you.
4. **Composable** — Works with transforms, Option, enums, nested structs, and fallible mapping.
5. **No heavy dependencies** — Only `syn`, `quote`, and `proc-macro2` at compile time. Zero runtime dependencies.

---

## Installation

Add `auto_morphic` to your dependencies in `Cargo.toml`:

```toml
[dependencies]
auto_morphic = "0.1"
```

### Optional Features

| Feature             | Description                                                 |
|---------------------|-------------------------------------------------------------|
| `serde-integration` | Reads `#[serde(rename = "...")]` as a fallback for renaming |

```toml
[dependencies]
auto_morphic = { version = "0.1", features = ["serde-integration"] }
```

---

## Quick Start

In five minutes, eliminate all manual conversion code between your structs.

### Step 1: Define your structs

```rust
use auto_morphic::{Morph, AutoMorph};

// Your domain model (database, business logic, etc.)
#[derive(Debug, Clone)]
struct User {
    id: i32,
    username: String,
    email: String,
    active: bool,
}

// Your DTO (API, serialization, etc.)
#[derive(Debug, Clone, Default, AutoMorph)]
#[map_to(User)]
struct UserDto {
    id: i32,
    #[map(name = "username")]
    name: String,
    #[map(name = "email")]
    email_address: String,
    #[map(name = "active")]
    is_active: bool,
}
```

### Step 2: Convert bidirectionally

```rust
let dto = UserDto {
id: 1,
name: "alice".to_string(),
email_address: "alice@example.com".to_string(),
is_active: true,
};

// DTO → Entity
let user: User = dto.morph_to();
assert_eq!(user.username, "alice");

// Entity → DTO
let dto_back: UserDto = UserDto::morph_from(user);
assert_eq!(dto_back.name, "alice");
```

### What happened?

The `AutoMorph` macro automatically generated:

```rust
// Generated code (equivalent)
impl Morph<User> for UserDto {
    fn morph_to(&self) -> User {
        User {
            id: self.id,
            username: self.name.clone(),
            email: self.email_address.clone(),
            active: self.is_active,
        }
    }

    fn morph_from(dto: User) -> Self {
        UserDto {
            id: dto.id,
            name: dto.username.clone(),
            email_address: dto.email.clone(),
            is_active: dto.active,
        }
    }
}
```

---

## Comparison with Alternatives

| Feature                  | **AutoMorphic**              | mapstruct-derive | serde            |
|--------------------------|------------------------------|------------------|------------------|
| **Bidirectional**        | ✅ `morph_to` + `morph_from` | ✅               | ❌               |
| **Transforms**           | ✅ `map_to_with`             | ✅               | ❌               |
| **Fallible mapping**     | ✅ `MorphResult`             | ❌               | ✅ (Deserialize) |
| **Enums**                | ✅ Unit variants             | ❌               | ✅               |
| **Multi-target**         | ✅ Multiple `#[map_to]`      | ❌               | ❌               |
| **Skip / Default**       | ✅                           | ✅               | Partial          |
| **Standard types**       | ✅ Implemented               | ❌               | ❌               |
| **Flatten**              | ✅                           | ❌               | ✅               |
| **Copy optimization**    | ✅                           | ❌               | —                |
| **Serde fallback**       | ✅ (feature)                 | —                | —                |
| **Runtime dependencies** | **Zero**                     | Zero             | serde + derive   |

---

## Attribute Guide

### `#[map_to(Type)]` — Struct level

Specifies the target type. Supports **multiple targets**:

```rust
#[derive(AutoMorph)]
#[map_to(UserDto)]
#[map_to(UserSummary)]
#[map_to(module::UserApiModel)]
struct User {
    /* ... */
}
```

Each target generates an independent `impl Morph<T>`.

### `#[map(...)]` — Field level

| Option                      | Description                                    | Example                                   |
|-----------------------------|------------------------------------------------|-------------------------------------------|
| `name = "field"`            | Renames the field in the target                | `#[map(name = "user_name")]`              |
| `map_to_with = fn`          | Transforms when converting **to** the target   | `#[map(map_to_with = uppercase)]`         |
| `map_from_with = fn`        | Transforms when converting **from** the target | `#[map(map_from_with = lowercase)]`       |
| `map_to_result_with = fn`   | Fallible version of `map_to_with`              | `#[map(map_to_result_with = parse_port)]` |
| `map_from_result_with = fn` | Fallible version of `map_from_with`            | `#[map(map_from_result_with = validate)]` |
| `skip`                      | Skips this field in both directions            | `#[map(skip)]`                            |
| `default`                   | Uses `Default::default()`                      | `#[map(default)]`                         |
| `default = expr`            | Custom default value                           | `#[map(default = 0)]`                     |
| `default_from = expr`       | Default only in `morph_from`                   | `#[map(default_from = true)]`             |
| `flatten`                   | Flatten nested struct                          | `#[map(flatten)]`                         |

---

## Usage Scenarios

### 1. Field Renaming

Names differ between layers (DB vs API):

```rust
#[derive(Clone, AutoMorph)]
#[map_to(User)]
struct UserDto {
    #[map(name = "user_name")]
    name: String,
    #[map(name = "email")]
    email_address: String,
}
```

### 2. Custom Transforms

Apply logic when converting:

```rust
fn uppercase(s: String) -> String { s.to_uppercase() }
fn to_titlecase(s: String) -> String { /* ... */ }

#[derive(Clone, AutoMorph)]
#[map_to(User)]
struct UserDto {
    #[map(name = "name", map_to_with = uppercase, map_from_with = to_titlecase)]
    title: String,
}
```

### 3. Skipping Fields

Some fields don't exist in the other structure:

```rust
#[derive(Clone, Default, AutoMorph)]
#[map_to(User)]
struct UserDto {
    name: String,
    #[map(skip)]
    password_hash: String,  // Never exposed in the DTO
    #[map(skip)]
    created_at: DateTime,   // Database only
}
```

> **Note:** `#[map(skip)]` requires both structs to derive `Default`.

### 4. Default Values

When a field exists in the target but not in the source:

```rust
#[derive(Clone, Default, AutoMorph)]
#[map_to(User)]
struct UserDto {
    name: String,
    #[map(default)]              // Uses Default::default()
    role: String,
    #[map(default = 0)]          // Literal
    score: i32,
    #[map(default_from = true)]  // Only in morph_from
    verified: bool,
}
```

### 5. Automatic Option<T>

Transparent conversion between optional and required:

```rust
#[derive(Clone)]
struct Profile {
    name: String,
    bio: String,        // Required
}

#[derive(Clone, AutoMorph)]
#[map_to(Profile)]
struct ProfileDto {
    name: String,
    bio: Option<String>, // Optional → unwrap_or_default()
}
```

- `Option<T>` → `T`: uses `unwrap_or_default()`
- `T` → `Option<T>`: wraps in `Some()`

### 6. Multi-target

A struct that converts to multiple destinations:

```rust
#[derive(Clone, AutoMorph)]
#[map_to(UserDto)]
#[map_to(UserSummary)]
#[map_to(UserApiModel)]
struct User {
    id: i32,
    name: String,
    email: String,
}
```

### 7. Nested Structs with Flatten

```rust
#[derive(Clone, Default)]
struct Address {
    city: String,
    zip: String,
}

#[derive(Clone, Default)]
struct User {
    name: String,
    address: Address,
}

#[derive(Clone, Default, AutoMorph)]
#[map_to(User)]
struct UserDto {
    name: String,
    #[map(flatten)]
    address: AddressDto,  // Morphs to Address and assigns
}
```

### 8. Enums

Mapping enums with unit variants:

```rust
#[derive(Debug, Clone, PartialEq)]
enum Status { Active, Inactive, Pending }

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Status)]
enum StatusDto {
    #[map(name = "Active")]
    Enabled,
    Disabled,
    #[map(name = "Pending")]
    Awaiting,
}
```

### 9. Fallible Mapping (MorphResult)

When a transformation can fail:

```rust
use auto_morphic::{MorphResult, MorphError, AutoMorph};

fn parse_port(s: String) -> Result<u16, std::num::ParseIntError> {
    s.parse()
}

#[derive(Clone, Default, AutoMorph)]
#[map_to(Server)]
struct ServerDto {
    host: String,
    #[map(name = "port", map_to_result_with = parse_port)]
    port_str: String,
}

// Usage
let dto = ServerDto {
host: "localhost".to_string(),
port_str: "8080".to_string(),
};

match dto.morph_to_result() {
Ok(server) => println ! ("Port: {}", server.port),
Err(e) => eprintln ! ("Error: {}", e),
}
```

### 10. Standard Types

Common conversions already implemented:

```rust
use auto_morphic::Morph;

// Integers
let u: u32 = 42i32.morph_to();
let i: i32 = u32::MAX.morph_to(); // Truncates with as!

// Floats
let f: f64 = 3.14f32.morph_to();

// Strings
let boxed: Box<str> = "hello".to_string().morph_to();

// Option
let s: String = Some("hi".to_string()).morph_to(); // unwrap_or_default
```

---

## Serde Integration

If you already use `#[serde(rename = "...")]`, enable it as a fallback:

```toml
[dependencies]
auto_morphic = { version = "0.1", features = ["serde-integration"] }
```

```rust
#[derive(AutoMorph)]
#[map_to(User)]
struct UserDto {
    #[serde(rename = "user_name")]
    name: String,  // Automatically maps to "user_name"
}
```

`#[map(name = "...")]` takes priority over `#[serde(rename = "...")]`.

---

## Performance

AutoMorphic delivers on its **zero cost** promise: the generated code is identical to hand-written code.

### Benchmark Results

| Benchmark               | AutoMorph | Manual | Difference     |
|-------------------------|-----------|--------|----------------|
| `morph_to` (6 fields)   | ~22 ns    | ~23 ns | **~0%**        |
| `morph_from` (6 fields) | ~23 ns    | —      | —              |
| `morph_to` (Option)     | ~22 ns    | —      | ~0 ns overhead |
| `morph_to` (20 fields)  | ~120 ns   | —      | ~6 ns/field    |

`Copy` types (`i32`, `bool`, `f64`, etc.) are accessed directly without `.clone()`.

Run the benchmarks:

```bash
cargo bench -p auto_morphic
```

---

## FAQ

### Can I use AutoMorphic without `Clone`?

No. The macro requires `Clone` on `String` and other non-`Copy` fields. `Copy` types (`i32`, `bool`, `f64`, etc.) are
automatically optimized and don't use `.clone()`.

### What happens if I forget to map a field?

The compiler will give you a clear error indicating which field is missing. The macro generates code that must compile —
if a field is not in `#[map(...)]` and doesn't exist in the target, compilation fails.

### Does it work with generic structs?

Yes, generic structs work with automatic trait bounds. Targets in `#[map_to]` must be concrete types (a limitation of
`syn` when parsing attributes).

### Can I combine `#[map(name)]` with `#[map(skip)]`?

Yes, but `skip` takes precedence: the field is completely omitted and `name` is ignored.

### How do I handle fields with the same name but different types?

Use `map_to_with` / `map_from_with` with a conversion function:

```rust
fn string_to_bool(s: String) -> bool { !s.is_empty() }
fn bool_to_string(b: bool) -> String { if b { "yes" } else { "no" }.to_string() }

#[map(name = "active", map_to_with = string_to_bool, map_from_with = bool_to_string)]
is_active: String,
```

### Is it thread-safe?

Yes. `morph_to()` takes `&self` and doesn't mutate global state. `morph_from()` is a pure function that receives `T` by
value.

---

## Roadmap

### Upcoming Versions

- **[v0.2]** `#[map(convention = "snake_case")]` — automatic renaming conventions
- **[v0.3]** Support for enums with data (tuple/struct variants)
- **[v0.4]** `#[map(target_option)]` — explicit control over Option behavior
- **[v0.5]** Mapping test generation with `#[cfg(test)]`
- **[v1.0]** API stability + complete documentation

### Long Term

- Integration with `tokio` for async mapping
- Derive for collections (`Vec<A>` ↔ `Vec<B>`)
- Compile-time schema validation

---

## Contributing

Contributions are welcome! Here's where to start:

1. **Report bugs** — [Issues](https://github.com/alexdev/auto_morphic/issues)
2. **Suggest features** — Discuss in Issues before opening PRs
3. **Improve documentation** — Doc PRs are always accepted
4. **Write tests** — Full coverage is a priority

### Local Development

```bash
# Clone and build
git clone https://github.com/alexdev/auto_morphic.git
cd auto_morphic
cargo build --workspace

# Run all tests
cargo test --workspace

# Check style and linting
cargo clippy --workspace -- -D warnings

# Run benchmarks
cargo bench -p auto_morphic
```

---

## Project Structure

```
auto_morphic/
├── auto_morphic/           # Main crate
│   ├── src/lib.rs          # Re-exports + 91 tests
│   ├── benches/            # Benchmarks with Criterion
│   └── examples/           # Demo app
├── auto_morphic_core/      # Traits and types
│   └── src/
│       ├── lib.rs          # Re-exports
│       └── morph.rs        # Morph, MorphResult, MorphError
└── auto_morphic_macro/     # Proc-macro
    └── src/lib.rs          # #[derive(AutoMorph)] implementation
```

---

## License

This project is licensed under **MIT** — see [LICENSE](LICENSE).

The code generated by the macro is yours. No telemetry, no hidden dependencies, no surprises.

---

> Made with ❤️ in Rust — because boilerplate shouldn't exist.
