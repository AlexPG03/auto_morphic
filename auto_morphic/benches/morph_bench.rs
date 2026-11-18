//! Benchmarks for AutoMorphic — compares generated code with manual equivalents.
//!
//! # What is Benchmarked
//!
//! 1. **Basic morph_to**: 6-field struct with field renaming (String + primitives).
//!    Compares AutoMorph-generated `morph_to()` with a hand-written function.
//! 2. **Basic morph_from**: Same struct, inverse direction.
//! 3. **Option morph**: Struct with `Option<String>` → `String` conversion
//!    using `unwrap_or_default()`.
//! 4. **Large struct**: 20-field struct (10 String + 5 i32 + 2 bool + 2 f64 + 1 i32)
//!    to measure how the generated code scales with field count.
//!
//! # How to Run
//!
//! ```bash
//! cargo bench -p auto_morphic
//! ```
//!
//! # Expected Results
//!
//! AutoMorph-generated code should be within ~5% of hand-written code,
//! since the macro produces the same field-by-field construction that
//! you would write manually. Copy types (i32, bool, f64) are accessed
//! directly without `.clone()`.

use auto_morphic_core::morph::Morph;
use auto_morphic_macro::AutoMorph;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

// ---------------------------------------------------------------------------
// 1. Basic struct: 6 fields (2 String, 2 i32, 1 bool, 1 f64) with renaming
// ---------------------------------------------------------------------------

/// Source entity — database model
#[derive(Debug, Clone, PartialEq)]
struct UserEntity {
    id: i32,
    username: String,
    email: String,
    age: i32,
    active: bool,
    score: f64,
}

/// DTO — API representation with renamed fields
#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(UserEntity)]
struct UserDto {
    id: i32,
    #[map(name = "username")]
    name: String,
    email: String,
    age: i32,
    active: bool,
    #[map(name = "score")]
    points: f64,
}

/// Hand-written equivalent of the generated `morph_to`
fn manual_morph_to(dto: &UserDto) -> UserEntity {
    UserEntity {
        id: dto.id,
        username: dto.name.clone(),
        email: dto.email.clone(),
        age: dto.age,
        active: dto.active,
        score: dto.points,
    }
}

/// Benchmark: morph_to — AutoMorph vs hand-written code
///
/// Measures the cost of converting a 6-field DTO to an Entity.
/// The AutoMorph version should match the manual version closely.
fn bench_morph_to(c: &mut Criterion) {
    let dto = UserDto {
        id: 42,
        name: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
        points: 99.5,
    };

    c.bench_function("morph_to (AutoMorph)", |b| {
        b.iter(|| black_box(&dto).morph_to())
    });

    c.bench_function("manual morph_to", |b| {
        b.iter(|| manual_morph_to(black_box(&dto)))
    });
}

/// Benchmark: morph_from — Entity back to DTO
fn bench_morph_from(c: &mut Criterion) {
    let entity = UserEntity {
        id: 42,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
        active: true,
        score: 99.5,
    };

    c.bench_function("morph_from (AutoMorph)", |b| {
        b.iter(|| UserDto::morph_from(black_box(entity.clone())))
    });
}

// ---------------------------------------------------------------------------
// 2. Option<T> → T conversion: unwrap_or_default overhead
// ---------------------------------------------------------------------------

/// Entity with required fields
#[derive(Debug, Clone, PartialEq, Default)]
struct Profile {
    name: String,
    bio: String,
    age: i32,
}

/// DTO with optional biography — maps to required `bio` via unwrap_or_default
#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Profile)]
struct ProfileDto {
    name: String,
    #[map(name = "bio")]
    biography: Option<String>,
    age: i32,
}

/// Benchmark: Option<T> → T conversion
///
/// Measures the overhead of `unwrap_or_default()` when the Option is `Some`.
/// The cost is minimal: one clone + unwrap_or_default call.
fn bench_option_morph(c: &mut Criterion) {
    let dto = ProfileDto {
        name: "bob".to_string(),
        biography: Some("Developer".to_string()),
        age: 25,
    };

    c.bench_function("morph_to with Option", |b| {
        b.iter(|| black_box(&dto).morph_to())
    });

    // Also benchmark morph_from: T → Option<T> (wraps in Some)
    let profile = Profile {
        name: "bob".to_string(),
        bio: "Developer".to_string(),
        age: 25,
    };

    c.bench_function("morph_from with Option", |b| {
        b.iter(|| ProfileDto::morph_from(black_box(profile.clone())))
    });
}

// ---------------------------------------------------------------------------
// 3. Large struct: 20 fields — scaling test
// ---------------------------------------------------------------------------

/// Large entity with 20 fields of mixed types
#[derive(Debug, Clone, PartialEq)]
struct LargeEntity {
    f1: String,
    f2: String,
    f3: String,
    f4: String,
    f5: String,
    f6: i32,
    f7: i32,
    f8: i32,
    f9: i32,
    f10: i32,
    f11: bool,
    f12: bool,
    f13: f64,
    f14: f64,
    f15: String,
    f16: String,
    f17: String,
    f18: String,
    f19: String,
    f20: i32,
}

/// DTO with identical fields — tests how AutoMorph scales with field count
#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(LargeEntity)]
struct LargeDto {
    f1: String,
    f2: String,
    f3: String,
    f4: String,
    f5: String,
    f6: i32,
    f7: i32,
    f8: i32,
    f9: i32,
    f10: i32,
    f11: bool,
    f12: bool,
    f13: f64,
    f14: f64,
    f15: String,
    f16: String,
    f17: String,
    f18: String,
    f19: String,
    f20: i32,
}

/// Benchmark: 20-field struct morph
///
/// Verifies that the generated code scales linearly with field count.
/// Copy types (i32, bool, f64) are accessed directly; String fields are cloned.
fn bench_large_struct(c: &mut Criterion) {
    let dto = LargeDto {
        f1: "a".to_string(),
        f2: "b".to_string(),
        f3: "c".to_string(),
        f4: "d".to_string(),
        f5: "e".to_string(),
        f6: 1,
        f7: 2,
        f8: 3,
        f9: 4,
        f10: 5,
        f11: true,
        f12: false,
        f13: 1.0,
        f14: 2.0,
        f15: "f".to_string(),
        f16: "g".to_string(),
        f17: "h".to_string(),
        f18: "i".to_string(),
        f19: "j".to_string(),
        f20: 6,
    };

    c.bench_function("morph_to (20 fields)", |b| {
        b.iter(|| black_box(&dto).morph_to())
    });

    let entity = LargeEntity {
        f1: "a".to_string(),
        f2: "b".to_string(),
        f3: "c".to_string(),
        f4: "d".to_string(),
        f5: "e".to_string(),
        f6: 1,
        f7: 2,
        f8: 3,
        f9: 4,
        f10: 5,
        f11: true,
        f12: false,
        f13: 1.0,
        f14: 2.0,
        f15: "f".to_string(),
        f16: "g".to_string(),
        f17: "h".to_string(),
        f18: "i".to_string(),
        f19: "j".to_string(),
        f20: 6,
    };

    c.bench_function("morph_from (20 fields)", |b| {
        b.iter(|| LargeDto::morph_from(black_box(entity.clone())))
    });
}

criterion_group!(
    benches,
    bench_morph_to,
    bench_morph_from,
    bench_option_morph,
    bench_large_struct
);
criterion_main!(benches);
