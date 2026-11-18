/// Trait for zero-cost, bidirectional struct-to-struct mapping.
///
/// `Morph<T>` defines two methods that convert between `Self` and `T`:
/// - [`morph_to`](Self::morph_to) — converts `&self` into `T`.
/// - [`morph_from`](Self::morph_from) — converts `T` into `Self`.
///
/// # Automatic Implementation
///
/// Use `#[derive(AutoMorph)]` from the `auto_morphic` crate to generate
/// the implementation automatically. The derive macro supports:
///
/// - Field renaming with `#[map(name = "...")]`
/// - Custom transforms with `#[map(map_to_with = fn, map_from_with = fn)]`
/// - Skipping fields with `#[map(skip)]`
/// - Default values with `#[map(default)]` or `#[map(default = expr)]`
/// - Automatic `Option<T>` ↔ `T` conversion
/// - Enum mapping with unit variants
/// - Flattening nested structs with `#[map(flatten)]`
/// - Multi-target: multiple `#[map_to(T)]` attributes
///
/// # Example
///
/// ```
/// use auto_morphic_core::Morph;
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct User { id: i32, name: String }
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct UserDto { id: i32, name: String }
///
/// impl Morph<User> for UserDto {
///     fn morph_to(&self) -> User {
///         User { id: self.id, name: self.name.clone() }
///     }
///     fn morph_from(t: User) -> Self {
///         UserDto { id: t.id, name: t.name }
///     }
/// }
///
/// let dto = UserDto { id: 1, name: "alice".to_string() };
/// let user: User = dto.morph_to();
/// assert_eq!(user.name, "alice");
///
/// let dto_back: UserDto = UserDto::morph_from(user);
/// assert_eq!(dto_back.name, "alice");
/// ```
///
/// # Standard Type Conversions
///
/// `Morph` is implemented for many standard type pairs out of the box:
///
/// | From | To |
/// |---|---|
/// | `i32` | `u32`, `i64`, `f32`, `f64` |
/// | `u32` | `i32`, `u64`, `f32`, `f64` |
/// | `i64` | `u64`, `f64` |
/// | `f64` | `i64`, `f32` |
/// | `bool` | `u8` |
/// | `String` | `Box<str>` |
/// | `Option<T>` | `T` (requires `T: Default + Clone`) |
///
/// # Fallible Morphing
///
/// For transformations that can fail (parsing, validation), use
/// [`MorphResult<T>`] with `morph_to_result()` / `morph_from_result()`.
pub trait Morph<T> {
    /// Convert `&self` into `T`.
    ///
    /// This method takes `&self` so the source value is not consumed.
    /// Fields are cloned or copied as needed.
    ///
    /// # Example
    ///
    /// ```
    /// use auto_morphic_core::Morph;
    ///
    /// #[derive(Clone)]
    /// struct Source { x: i32 }
    /// #[derive(Clone)]
    /// struct Target { x: i32 }
    ///
    /// impl Morph<Target> for Source {
    ///     fn morph_to(&self) -> Target { Target { x: self.x } }
    ///     fn morph_from(t: Target) -> Self { Source { x: t.x } }
    /// }
    ///
    /// let src = Source { x: 42 };
    /// let tgt: Target = src.morph_to();
    /// assert_eq!(tgt.x, 42);
    /// ```
    fn morph_to(&self) -> T;

    /// Convert `T` into `Self`.
    ///
    /// This is the inverse of [`morph_to`](Self::morph_to).
    /// The source value `t` is consumed (moved).
    ///
    /// # Example
    ///
    /// ```
    /// use auto_morphic_core::Morph;
    ///
    /// #[derive(Clone)]
    /// struct Source { x: i32 }
    /// #[derive(Clone)]
    /// struct Target { x: i32 }
    ///
    /// impl Morph<Target> for Source {
    ///     fn morph_to(&self) -> Target { Target { x: self.x } }
    ///     fn morph_from(t: Target) -> Self { Source { x: t.x } }
    /// }
    ///
    /// let tgt = Target { x: 42 };
    /// let src: Source = Source::morph_from(tgt);
    /// assert_eq!(src.x, 42);
    /// ```
    fn morph_from(t: T) -> Self;
}

// ============================================================================
// Standard type implementations
// ============================================================================

// String <-> Box<str>
impl Morph<String> for Box<str> {
    fn morph_to(&self) -> String {
        self.to_string()
    }
    fn morph_from(t: String) -> Self {
        t.into_boxed_str()
    }
}

impl Morph<Box<str>> for String {
    fn morph_to(&self) -> Box<str> {
        self.clone().into_boxed_str()
    }
    fn morph_from(t: Box<str>) -> Self {
        t.to_string()
    }
}

// Integer conversions
macro_rules! impl_morph_int {
    ($from:ty, $to:ty) => {
        impl Morph<$to> for $from {
            fn morph_to(&self) -> $to {
                *self as $to
            }
            fn morph_from(t: $to) -> Self {
                t as $from
            }
        }
    };
}

// Signed <-> Unsigned
impl_morph_int!(i8, u8);
impl_morph_int!(u8, i8);
impl_morph_int!(i16, u16);
impl_morph_int!(u16, i16);
impl_morph_int!(i32, u32);
impl_morph_int!(u32, i32);
impl_morph_int!(i64, u64);
impl_morph_int!(u64, i64);
impl_morph_int!(i128, u128);
impl_morph_int!(u128, i128);
impl_morph_int!(isize, usize);
impl_morph_int!(usize, isize);

// Same-width signed conversions
impl_morph_int!(i8, i16);
impl_morph_int!(i16, i8);
impl_morph_int!(i8, i32);
impl_morph_int!(i32, i8);
impl_morph_int!(i8, i64);
impl_morph_int!(i64, i8);
impl_morph_int!(i16, i32);
impl_morph_int!(i32, i16);
impl_morph_int!(i16, i64);
impl_morph_int!(i64, i16);
impl_morph_int!(i32, i64);
impl_morph_int!(i64, i32);

// Same-width unsigned conversions
impl_morph_int!(u8, u16);
impl_morph_int!(u16, u8);
impl_morph_int!(u8, u32);
impl_morph_int!(u32, u8);
impl_morph_int!(u8, u64);
impl_morph_int!(u64, u8);
impl_morph_int!(u16, u32);
impl_morph_int!(u32, u16);
impl_morph_int!(u16, u64);
impl_morph_int!(u64, u16);
impl_morph_int!(u32, u64);
impl_morph_int!(u64, u32);

// Float <-> Int
impl_morph_int!(i32, f32);
impl_morph_int!(f32, i32);
impl_morph_int!(i64, f64);
impl_morph_int!(f64, i64);
impl_morph_int!(u32, f32);
impl_morph_int!(f32, u32);
impl_morph_int!(u64, f64);
impl_morph_int!(f64, u64);
impl_morph_int!(f32, f64);
impl_morph_int!(f64, f32);

// Bool <-> u8 (cannot use `as` cast, need conversion)
impl Morph<u8> for bool {
    fn morph_to(&self) -> u8 {
        if *self { 1 } else { 0 }
    }
    fn morph_from(t: u8) -> Self {
        t != 0
    }
}

impl Morph<bool> for u8 {
    fn morph_to(&self) -> bool {
        *self != 0
    }
    fn morph_from(t: bool) -> Self {
        if t { 1 } else { 0 }
    }
}

// Option<T> <-> T (for types that implement Default + Clone)
impl<T: Default + Clone> Morph<T> for Option<T> {
    fn morph_to(&self) -> T {
        self.clone().unwrap_or_default()
    }
    fn morph_from(t: T) -> Self {
        Some(t)
    }
}

impl<T: Default + Clone> Morph<Option<T>> for T {
    fn morph_to(&self) -> Option<T> {
        Some(self.clone())
    }
    fn morph_from(t: Option<T>) -> Self {
        t.unwrap_or_default()
    }
}

// Vec<T> <-> Vec<T> (identity, useful for generic bounds)
// Note: Vec<&[T]> omitted due to lifetime issues with the Morph trait

// ============================================================================
// MorphResult trait — fallible morphing
// ============================================================================

/// Error type for fallible morph operations.
///
/// `MorphError` is returned by [`MorphResult::morph_to_result`] and
/// [`MorphResult::morph_from_result`] when a field transformation fails.
/// It carries both a human-readable message and an optional field name
/// so the caller can pinpoint which field caused the error.
///
/// # Example
///
/// ```
/// use auto_morphic_core::MorphError;
///
/// // Create a generic error
/// let err = MorphError::new("parsing failed");
/// assert!(err.field.is_none());
///
/// // Create a per-field error
/// let err = MorphError::field("port", "invalid number");
/// assert_eq!(err.field.as_deref(), Some("port"));
/// ```
#[derive(Debug, Clone)]
pub struct MorphError {
    /// Human-readable description of the error.
    pub message: String,
    /// The field name where the error occurred, if applicable.
    pub field: Option<String>,
}

impl MorphError {
    /// Create a new `MorphError` with a message and no field name.
    ///
    /// Use this for errors that are not tied to a specific field.
    ///
    /// # Example
    ///
    /// ```
    /// use auto_morphic_core::MorphError;
    /// let err = MorphError::new("configuration invalid");
    /// assert_eq!(err.message, "configuration invalid");
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: None,
        }
    }

    /// Create a new `MorphError` associated with a specific field.
    ///
    /// The `field` argument identifies which struct field failed to convert.
    /// This is the preferred constructor when generating errors inside
    /// `morph_to_result()` or `morph_from_result()` implementations.
    ///
    /// # Example
    ///
    /// ```
    /// use auto_morphic_core::MorphError;
    /// let err = MorphError::field("age", "must be positive");
    /// assert_eq!(err.to_string(), "MorphError in field 'age': must be positive");
    /// ```
    pub fn field(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            field: Some(field.into()),
        }
    }
}

impl std::fmt::Display for MorphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref field) = self.field {
            write!(f, "MorphError in field '{}': {}", field, self.message)
        } else {
            write!(f, "MorphError: {}", self.message)
        }
    }
}

impl std::error::Error for MorphError {}

/// Trait for fallible, bidirectional struct-to-struct mapping.
///
/// `MorphResult<T>` is the fallible counterpart of [`Morph<T>`](crate::Morph).
/// Use it when a field transformation can fail — for example, parsing a string
/// into a number, validating a range, or deserializing JSON.
///
/// # Automatic Generation
///
/// The `#[derive(AutoMorph)]` macro generates `impl MorphResult<T>` automatically
/// when it detects `map_to_result_with` or `map_from_result_with` attributes on
/// any field. Fields without a `*_result_with` attribute fall back to the
/// corresponding `*_with` function or to direct field access.
///
/// # Example: Manual Implementation
///
/// ```
/// use auto_morphic_core::{MorphResult, MorphError};
///
/// #[derive(Debug, Clone)]
/// struct Config { port: u16 }
///
/// #[derive(Debug, Clone)]
/// struct ConfigInput { port_str: String }
///
/// impl MorphResult<Config> for ConfigInput {
///     fn morph_to_result(&self) -> Result<Config, MorphError> {
///         Ok(Config {
///             port: self.port_str.parse::<u16>()
///                 .map_err(|e: std::num::ParseIntError| MorphError::field("port", e.to_string()))?,
///         })
///     }
///
///     fn morph_from_result(t: Config) -> Result<Self, MorphError> {
///         Ok(Self {
///             port_str: t.port.to_string(),
///         })
///     }
/// }
///
/// let input = ConfigInput { port_str: "8080".to_string() };
/// let config: Config = input.morph_to_result().unwrap();
/// assert_eq!(config.port, 8080);
///
/// let bad = ConfigInput { port_str: "abc".to_string() };
/// assert!(bad.morph_to_result().is_err());
/// ```
///
/// # Example: Derive Macro
///
/// ```ignore
/// fn parse_port(s: String) -> Result<u16, std::num::ParseIntError> {
///     s.parse()
/// }
///
/// #[derive(AutoMorph)]
/// #[map_to(Config)]
/// struct ConfigInput {
///     #[map(name = "port", map_to_result_with = parse_port)]
///     port_str: String,
/// }
/// // impl MorphResult<Config> for ConfigInput is generated automatically
/// ```
///
/// # Error Propagation
///
/// Each field's transformation wraps errors with [`MorphError::field`] so the
/// caller knows which field failed. Use `?` inside the generated code to
/// short-circuit on the first error.
pub trait MorphResult<T> {
    /// Convert `&self` into `T`, returning an error if any field transformation fails.
    ///
    /// Returns `Ok(T)` on success or `Err(MorphError)` with the field name and
    /// error message on failure.
    ///
    /// # Example
    ///
    /// ```
    /// use auto_morphic_core::{MorphResult, MorphError};
    ///
    /// #[derive(Clone)]
    /// struct Target { x: u16 }
    /// #[derive(Clone)]
    /// struct Source { x_str: String }
    ///
    /// impl MorphResult<Target> for Source {
    ///     fn morph_to_result(&self) -> Result<Target, MorphError> {
    ///         Ok(Target {
    ///             x: self.x_str.parse().map_err(|e: std::num::ParseIntError| MorphError::field("x", e.to_string()))?,
    ///         })
    ///     }
    ///     fn morph_from_result(t: Target) -> Result<Self, MorphError> {
    ///         Ok(Self { x_str: t.x.to_string() })
    ///     }
    /// }
    ///
    /// let src = Source { x_str: "42".to_string() };
    /// let tgt: Target = src.morph_to_result().unwrap();
    /// assert_eq!(tgt.x, 42);
    /// ```
    fn morph_to_result(&self) -> Result<T, MorphError>;

    /// Convert `T` into `Self`, returning an error if any field transformation fails.
    ///
    /// This is the inverse of [`morph_to_result`](Self::morph_to_result).
    /// The source value `t` is consumed (moved).
    fn morph_from_result(t: T) -> Result<Self, MorphError>
    where
        Self: Sized;
}

// Note: No blanket impl of MorphResult for Morph, because the macro
// may generate its own MorphResult impl. Users who want fallible morphing
// should implement MorphResult manually.
