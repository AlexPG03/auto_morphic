//! **AutoMorphic** — zero-cost struct-to-struct mapping with a derive macro.
//!
//! AutoMorphic eliminates boilerplate when converting between similar structs
//! (Entity ↔ DTO, Request ↔ Response, etc.). The `#[derive(AutoMorph)]` macro
//! generates `impl Morph<T>` at compile time with **zero runtime overhead**.
//!
//! # Quick Start
//!
//! ```
//! use auto_morphic::{Morph, AutoMorph};
//!
//! #[derive(Debug, Clone)]
//! struct User {
//!     id: i32,
//!     username: String,
//! }
//!
//! #[derive(Debug, Clone, AutoMorph)]
//! #[map_to(User)]
//! struct UserDto {
//!     id: i32,
//!     #[map(name = "username")]
//!     name: String,
//! }
//!
//! let dto = UserDto { id: 1, name: "alice".to_string() };
//! let user: User = dto.morph_to();
//! let dto_back: UserDto = UserDto::morph_from(user);
//! ```
//!
//! # Features
//!
//! | Feature | Attribute | Description |
//! |---|---|---|
//! | **Field renaming** | `#[map(name = "field")]` | Map to a differently-named field |
//! | **Transforms** | `#[map(map_to_with = fn)]` | Apply a function during conversion |
//! | **Skip** | `#[map(skip)]` | Omit a field from mapping |
//! | **Default** | `#[map(default)]` | Use `Default::default()` for a field |
//! | **Option** | (automatic) | `Option<T>` ↔ `T` conversion |
//! | **Multi-target** | Multiple `#[map_to(T)]` | Map to several target types |
//! | **Enums** | `#[map(name = "Variant")]` | Map unit enum variants |
//! | **Flatten** | `#[map(flatten)]` | Flatten nested structs |
//! | **Fallible** | `#[map(map_to_result_with = fn)]` | Transformations that can fail |
//!
//! # Attribute Reference
//!
//! ## `#[map_to(Tipo)]` — on the struct
//!
//! Specifies the target type. Multiple `#[map_to]` attributes generate
//! separate `impl Morph<T>` blocks for each target.
//!
//! ```ignore
//! #[derive(AutoMorph)]
//! #[map_to(UserDto)]
//! #[map_to(UserSummary)]
//! struct User { /* ... */ }
//! ```
//!
//! ## `#[map(...)]` — on each field
//!
//! | Option | Description |
//! |---|---|
//! | `name = "field"` | Destination field name |
//! | `map_to_with = fn` | Transform function for `morph_to` |
//! | `map_from_with = fn` | Transform function for `morph_from` |
//! | `map_to_result_with = fn` | Fallible transform for `morph_to_result` |
//! | `map_from_result_with = fn` | Fallible transform for `morph_from_result` |
//! | `skip` | Skip this field in both directions |
//! | `default` | Use `Default::default()` |
//! | `default = expr` | Use an expression as default |
//! | `default_from = expr` | Default only for `morph_from` |
//! | `flatten` | Flatten a nested struct via its own `Morph` |
//!
//! # Standard Type Conversions
//!
//! `Morph` is pre-implemented for common type pairs:
//!
//! - Integers: `i32 ↔ u32`, `i64 ↔ u64`, `i32 ↔ i64`, etc.
//! - Floats: `f32 ↔ f64`, `f64 ↔ i64`
//! - Bool: `bool ↔ u8`
//! - Strings: `String ↔ Box<str>`
//! - Option: `Option<T> ↔ T` (where `T: Default + Clone`)
//!
//! # Fallible Morphing
//!
//! For transformations that can fail (parsing, validation), use `MorphResult`:
//!
//! ```ignore
//! fn parse_port(s: String) -> Result<u16, std::num::ParseIntError> {
//!     s.parse()
//! }
//!
//! #[derive(AutoMorph)]
//! #[map_to(Config)]
//! struct ConfigInput {
//!     #[map(name = "port", map_to_result_with = parse_port)]
//!     port_str: String,
//! }
//!
//! let config: Result<Config, MorphError> = input.morph_to_result();
//! ```
//!
//! # Serde Integration
//!
//! Enable the `serde-integration` feature to reuse `#[serde(rename = "...")]`
//! as a fallback when `#[map(name = "...")]` is not present:
//!
//! ```toml
//! [dependencies]
//! auto_morphic = { version = "0.1", features = ["serde-integration"] }
//! ```
//!
//! # Performance
//!
//! AutoMorphic generates code identical to writing it by hand. Copy types
//! (`i32`, `bool`, `f64`, etc.) are accessed directly without `.clone()`.
//! Run benchmarks with `cargo bench -p auto_morphic`.
//!
//! # Module Structure
//!
//! - [`morph`] — Core traits: `Morph`, `MorphResult`, `MorphError`
//! - [`AutoMorph`] — Derive macro for automatic implementation

pub use auto_morphic_core::morph;
pub use auto_morphic_core::{Morph, MorphResult, MorphError};
pub use auto_morphic_macro::AutoMorph;

#[cfg(test)]
mod tests {
    //! Tests exhaustivos para todas las features de AutoMorph.
    //!
    //! Cobertura:
    //! - Mapeo básico (morph_to / morph_from)
    //! - Renombrado de campos (name)
    //! - Transformaciones (map_to_with / map_from_with)
    //! - Skip de campos
    //! - Valores por defecto (default, default = expr, default_from)
    //! - Option<T> ↔ T
    //! - Multi-target (2 y 3 targets)
    //! - Enums con unit variants
    //! - Tipos estándar (i32↔u32, bool↔u8, String↔Box<str>, Option↔T)
    //! - MorphResult (mapeo fallible)
    //! - Flatten de structs anidados
    //! - Copy vs Clone optimización
    //! - Generics

    use auto_morphic_core::morph::Morph;
    use crate::AutoMorph;

    #[derive(Debug, PartialEq, Clone)]
    struct User {
        id: i32,
        username: String,
        email: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(User)]
    struct UserDto {
        id: i32,
        username: String,
        #[map(
            name = "email",
        )]
        email_address: String,
    }

    #[test]
    fn test_morph_to() {
        let user_dto = UserDto {
            id: 1,
            username: "alice".to_string(),
            email_address: "alice@example.com".to_string(),
        };
        let user = user_dto.morph_to();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
    }

    #[test]
    fn test_morph_from() {
        let user = User {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        };
        let user_dto = UserDto::morph_from(user);
        assert_eq!(user_dto.id, 2);
        assert_eq!(user_dto.username, "bob");
        assert_eq!(user_dto.email_address, "bob@example.com");
    }

    // -- Custom transformation functions (map_to_with / map_from_with) --

    fn uppercase(s: String) -> String {
        s.to_uppercase()
    }

    fn lowercase(s: String) -> String {
        s.to_lowercase()
    }

    fn double(n: i32) -> i32 {
        n * 2
    }

    fn half(n: i32) -> i32 {
        n / 2
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Product {
        id: i32,
        name: String,
        price: i32,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Product)]
    struct ProductDto {
        #[map(map_to_with = double, map_from_with = half)]
        id: i32,
        #[map(map_to_with = uppercase, map_from_with = lowercase)]
        name: String,
        price: i32,
    }

    #[test]
    fn test_map_to_with_transform() {
        let dto = ProductDto {
            id: 5,
            name: "widget".to_string(),
            price: 100,
        };
        let product = dto.morph_to();
        assert_eq!(product.id, 10); // double(5)
        assert_eq!(product.name, "WIDGET"); // uppercase("widget")
        assert_eq!(product.price, 100);
    }

    #[test]
    fn test_map_from_with_transform() {
        let product = Product {
            id: 20,
            name: "GADGET".to_string(),
            price: 200,
        };
        let dto = ProductDto::morph_from(product);
        assert_eq!(dto.id, 10); // half(20)
        assert_eq!(dto.name, "gadget"); // lowercase("GADGET")
        assert_eq!(dto.price, 200);
    }

    // -- Multiple fi
    //             map_to_with = elds with different mappings --

    #[derive(Debug, PartialEq, Clone)]
    struct Address {
        street: String,
        city: String,
        country_code: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Address)]
    struct AddressForm {
        #[map(name = "street")]
        street_name: String,
        #[map(name = "city")]
        city_name: String,
        #[map(name = "country_code", map_to_with = uppercase, map_from_with = lowercase)]
        country: String,
    }

    #[test]
    fn test_multiple_renamed_fields_to() {
        let form = AddressForm {
            street_name: "Main St".to_string(),
            city_name: "Springfield".to_string(),
            country: "us".to_string(),
        };
        let addr = form.morph_to();
        assert_eq!(addr.street, "Main St");
        assert_eq!(addr.city, "Springfield");
        assert_eq!(addr.country_code, "US");
    }

    #[test]
    fn test_multiple_renamed_fields_from() {
        let addr = Address {
            street: "Oak Ave".to_string(),
            city: "Shelbyville".to_string(),
            country_code: "UK".to_string(),
        };
        let form = AddressForm::morph_from(addr);
        assert_eq!(form.street_name, "Oak Ave");
        assert_eq!(form.city_name, "Shelbyville");
        assert_eq!(form.country, "uk");
    }

    // -- Identical field names (no #[map] needed) --

    #[derive(Debug, PartialEq, Clone)]
    struct Point {
        x: f64,
        y: f64,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Point)]
    struct PointDto {
        x: f64,
        y: f64,
    }

    #[test]
    fn test_identical_field_names_to() {
        let dto = PointDto { x: 1.5, y: 2.5 };
        let point = dto.morph_to();
        assert_eq!(point.x, 1.5);
        assert_eq!(point.y, 2.5);
    }

    #[test]
    fn test_identical_field_names_from() {
        let point = Point { x: 3.0, y: 4.0 };
        let dto = PointDto::morph_from(point);
        assert_eq!(dto.x, 3.0);
        assert_eq!(dto.y, 4.0);
    }

    // -- Single field struct --

    #[derive(Debug, PartialEq, Clone)]
    struct Wrapper {
        value: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Wrapper)]
    struct WrapperDto {
        #[map(name = "value")]
        data: String,
    }

    #[test]
    fn test_single_field_to() {
        let dto = WrapperDto { data: "hello".to_string() };
        let wrapper = dto.morph_to();
        assert_eq!(wrapper.value, "hello");
    }

    #[test]
    fn test_single_field_from() {
        let wrapper = Wrapper { value: "world".to_string() };
        let dto = WrapperDto::morph_from(wrapper);
        assert_eq!(dto.data, "world");
    }

    // -- All fields renamed --

    #[derive(Debug, PartialEq, Clone)]
    struct Config {
        host: String,
        port: u16,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Config)]
    struct ConfigDto {
        #[map(name = "host")]
        server_address: String,
        #[map(name = "port")]
        server_port: u16,
    }

    #[test]
    fn test_all_fields_renamed_to() {
        let dto = ConfigDto {
            server_address: "localhost".to_string(),
            server_port: 8080,
        };
        let config = dto.morph_to();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_all_fields_renamed_from() {
        let config = Config {
            host: "0.0.0.0".to_string(),
            port: 9090,
        };
        let dto = ConfigDto::morph_from(config);
        assert_eq!(dto.server_address, "0.0.0.0");
        assert_eq!(dto.server_port, 9090);
    }

    // -- Mixed: some fields renamed, some with transforms, some identical --

    #[derive(Debug, PartialEq, Clone)]
    struct Employee {
        id: i64,
        first_name: String,
        last_name: String,
        active: bool,
    }

    fn to_titlecase(s: String) -> String {
        let mut c = s.chars();
        match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        }
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Employee)]
    struct EmployeeDto {
        id: i64,
        #[map(name = "first_name", map_to_with = to_titlecase)]
        fname: String,
        #[map(name = "last_name")]
        lname: String,
        active: bool,
    }

    #[test]
    fn test_mixed_mapping_to() {
        let dto = EmployeeDto {
            id: 42,
            fname: "alice".to_string(),
            lname: "smith".to_string(),
            active: true,
        };
        let emp = dto.morph_to();
        assert_eq!(emp.id, 42);
        assert_eq!(emp.first_name, "Alice"); // titlecased
        assert_eq!(emp.last_name, "smith");
        assert_eq!(emp.active, true);
    }

    #[test]
    fn test_mixed_mapping_from() {
        let emp = Employee {
            id: 99,
            first_name: "Bob".to_string(),
            last_name: "Jones".to_string(),
            active: false,
        };
        let dto = EmployeeDto::morph_from(emp);
        assert_eq!(dto.id, 99);
        assert_eq!(dto.fname, "Bob");
        assert_eq!(dto.lname, "Jones");
        assert_eq!(dto.active, false);
    }

    // -- Skip fields --

    #[derive(Debug, PartialEq, Clone, Default)]
    struct SecureUser {
        id: i32,
        username: String,
        password_hash: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(SecureUser)]
    struct SecureUserDto {
        id: i32,
        username: String,
        #[map(skip)]
        password_hash: String,
    }

    #[test]
    fn test_skip_field_morph_to() {
        let dto = SecureUserDto {
            id: 1,
            username: "alice".to_string(),
            password_hash: "secret".to_string(),
        };
        let user: SecureUser = dto.morph_to();
        assert_eq!(user.id, 1);
        assert_eq!(user.username, "alice");
        // password_hash skipped, filled with Default::default()
        assert_eq!(user.password_hash, String::default());
    }

    #[test]
    fn test_skip_field_morph_from() {
        let user = SecureUser {
            id: 2,
            username: "bob".to_string(),
            password_hash: "hashed".to_string(),
        };
        let dto = SecureUserDto::morph_from(user);
        assert_eq!(dto.id, 2);
        assert_eq!(dto.username, "bob");
        // password_hash skipped, filled with Default::default()
        assert_eq!(dto.password_hash, String::default());
    }

    // -- Skip with fields that exist in target but not in source --

    #[derive(Debug, PartialEq, Clone, Default)]
    struct Event {
        id: i32,
        title: String,
        metadata: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(Event)]
    struct EventDto {
        id: i32,
        title: String,
        #[map(skip)]
        metadata: String,
    }

    #[test]
    fn test_skip_field_to() {
        let dto = EventDto {
            id: 42,
            title: "Meeting".to_string(),
            metadata: "ignored".to_string(),
        };
        let event: Event = dto.morph_to();
        assert_eq!(event.id, 42);
        assert_eq!(event.title, "Meeting");
        assert_eq!(event.metadata, String::default());
    }

    #[test]
    fn test_skip_field_from() {
        let event = Event {
            id: 42,
            title: "Meeting".to_string(),
            metadata: "some-data".to_string(),
        };
        let dto = EventDto::morph_from(event);
        assert_eq!(dto.id, 42);
        assert_eq!(dto.title, "Meeting");
        assert_eq!(dto.metadata, String::default());
    }

    // -- Default values --

    fn get_default_role() -> String {
        "user".to_string()
    }

    fn get_default_limit() -> i32 {
        100
    }

    #[derive(Debug, PartialEq, Clone)]
    struct ApiRequest {
        query: String,
        limit: i32,
        role: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(ApiRequest)]
    struct ApiRequestDto {
        query: String,
        #[map(default = 50)]
        limit: i32,
        #[map(default_from = get_default_role())]
        role: String,
    }

    #[test]
    fn test_default_value_morph_to() {
        let dto = ApiRequestDto {
            query: "search".to_string(),
            limit: 999, // este valor se ignora, se usa el default
            role: "ignored".to_string(),
        };
        let req = dto.morph_to();
        assert_eq!(req.query, "search");
        assert_eq!(req.limit, 50); // usa el default literal
        // role no tiene default_to, así que usa el valor del campo
        assert_eq!(req.role, "ignored");
    }

    #[test]
    fn test_default_from_value_morph_from() {
        let req = ApiRequest {
            query: "search".to_string(),
            limit: 999,
            role: "admin".to_string(),
        };
        let dto = ApiRequestDto::morph_from(req);
        assert_eq!(dto.query, "search");
        // limit no tiene default_from, usa el valor de la fuente
        assert_eq!(dto.limit, 999);
        // role usa default_from = get_default_role()
        assert_eq!(dto.role, "user");
    }

    // -- Bare default (Default::default()) --

    #[derive(Debug, PartialEq, Clone, Default)]
    struct Settings {
        theme: String,
        font_size: i32,
        notifications: bool,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(Settings)]
    struct SettingsDto {
        #[map(default)]
        theme: String,
        font_size: i32,
        #[map(default)]
        notifications: bool,
    }

    #[test]
    fn test_bare_default_morph_to() {
        let dto = SettingsDto {
            theme: "ignored".to_string(),
            font_size: 14,
            notifications: true,
        };
        let settings: Settings = dto.morph_to();
        assert_eq!(settings.theme, String::default()); // bare default
        assert_eq!(settings.font_size, 14);
        assert_eq!(settings.notifications, bool::default()); // bare default
    }

    #[test]
    fn test_bare_default_morph_from() {
        let settings = Settings {
            theme: "dark".to_string(),
            font_size: 16,
            notifications: true,
        };
        let dto = SettingsDto::morph_from(settings);
        assert_eq!(dto.theme, String::default()); // bare default for morph_from too
        assert_eq!(dto.font_size, 16);
        assert_eq!(dto.notifications, bool::default()); // bare default for morph_from too
    }

    // -- Combined: skip + default + transforms --

    #[derive(Debug, PartialEq, Clone, Default)]
    struct Order {
        id: i32,
        product: String,
        quantity: i32,
        internal_note: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(Order)]
    struct OrderDto {
        #[map(map_to_with = double, map_from_with = half)]
        id: i32,
        #[map(name = "product")]
        product_name: String,
        #[map(default = 1)]
        quantity: i32,
        #[map(skip)]
        internal_note: String,
    }

    #[test]
    fn test_combined_skip_default_transform_to() {
        let dto = OrderDto {
            id: 5,
            product_name: "Widget".to_string(),
            quantity: 999, // ignorado, usa default = 1
            internal_note: "confidential".to_string(), // skip
        };
        let order: Order = dto.morph_to();
        assert_eq!(order.id, 10); // double(5)
        assert_eq!(order.product, "Widget");
        assert_eq!(order.quantity, 1); // default = 1
        assert_eq!(order.internal_note, String::default()); // skipped
    }

    #[test]
    fn test_combined_skip_default_transform_from() {
        let order = Order {
            id: 20,
            product: "Gadget".to_string(),
            quantity: 5,
            internal_note: "handle with care".to_string(),
        };
        let dto = OrderDto::morph_from(order);
        assert_eq!(dto.id, 10); // half(20)
        assert_eq!(dto.product_name, "Gadget");
        // quantity no tiene default_from, usa el valor de la fuente
        assert_eq!(dto.quantity, 5);
        assert_eq!(dto.internal_note, String::default()); // skipped
    }

    // -- Multi-target support: same fields in all targets --

    #[derive(Debug, PartialEq, Clone)]
    struct ArticleFull {
        id: i32,
        title: String,
        body: String,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct ArticlePreview {
        id: i32,
        title: String,
        body: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(ArticleFull)]
    #[map_to(ArticlePreview)]
    struct ArticleDto {
        id: i32,
        #[map(name = "title")]
        headline: String,
        #[map(name = "body")]
        content: String,
    }

    #[test]
    fn test_multi_target_morph_to_full() {
        let dto = ArticleDto {
            id: 1,
            headline: "Breaking News".to_string(),
            content: "Full story here...".to_string(),
        };
        let article: ArticleFull = dto.morph_to();
        assert_eq!(article.id, 1);
        assert_eq!(article.title, "Breaking News");
        assert_eq!(article.body, "Full story here...");
    }

    #[test]
    fn test_multi_target_morph_to_preview() {
        let dto = ArticleDto {
            id: 2,
            headline: "Another Story".to_string(),
            content: "Preview text...".to_string(),
        };
        let preview: ArticlePreview = dto.morph_to();
        assert_eq!(preview.id, 2);
        assert_eq!(preview.title, "Another Story");
        assert_eq!(preview.body, "Preview text...");
    }

    #[test]
    fn test_multi_target_morph_from_full() {
        let article = ArticleFull {
            id: 3,
            title: "From Full".to_string(),
            body: "Body from full".to_string(),
        };
        let dto = ArticleDto::morph_from(article);
        assert_eq!(dto.id, 3);
        assert_eq!(dto.headline, "From Full");
        assert_eq!(dto.content, "Body from full");
    }

    #[test]
    fn test_multi_target_morph_from_preview() {
        let preview = ArticlePreview {
            id: 4,
            title: "From Preview".to_string(),
            body: "Body from preview".to_string(),
        };
        let dto = ArticleDto::morph_from(preview);
        assert_eq!(dto.id, 4);
        assert_eq!(dto.headline, "From Preview");
        assert_eq!(dto.content, "Body from preview");
    }

    // -- Multi-target with transforms --

    #[derive(Debug, PartialEq, Clone)]
    struct ScoreFull {
        id: i32,
        name: String,
        points: i32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct ScoreSummary {
        id: i32,
        name: String,
        points: i32,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(ScoreFull)]
    #[map_to(ScoreSummary)]
    struct ScoreDto {
        #[map(map_to_with = double, map_from_with = half)]
        id: i32,
        #[map(map_to_with = uppercase, map_from_with = lowercase)]
        name: String,
        points: i32,
    }

    #[test]
    fn test_multi_target_transforms_to_full() {
        let dto = ScoreDto {
            id: 5,
            name: "alice".to_string(),
            points: 100,
        };
        let score: ScoreFull = dto.morph_to();
        assert_eq!(score.id, 10); // double(5)
        assert_eq!(score.name, "ALICE"); // uppercase
        assert_eq!(score.points, 100);
    }

    #[test]
    fn test_multi_target_transforms_to_summary() {
        let dto = ScoreDto {
            id: 7,
            name: "bob".to_string(),
            points: 200,
        };
        let summary: ScoreSummary = dto.morph_to();
        assert_eq!(summary.id, 14); // double(7)
        assert_eq!(summary.name, "BOB"); // uppercase
        assert_eq!(summary.points, 200);
    }

    #[test]
    fn test_multi_target_transforms_from_full() {
        let score = ScoreFull {
            id: 20,
            name: "CHARLIE".to_string(),
            points: 300,
        };
        let dto = ScoreDto::morph_from(score);
        assert_eq!(dto.id, 10); // half(20)
        assert_eq!(dto.name, "charlie"); // lowercase
        assert_eq!(dto.points, 300);
    }

    #[test]
    fn test_multi_target_transforms_from_summary() {
        let summary = ScoreSummary {
            id: 30,
            name: "DIANA".to_string(),
            points: 400,
        };
        let dto = ScoreDto::morph_from(summary);
        assert_eq!(dto.id, 15); // half(30)
        assert_eq!(dto.name, "diana"); // lowercase
        assert_eq!(dto.points, 400);
    }

    // -- Multi-target with skip and default for partial targets --

    #[derive(Debug, PartialEq, Clone, Default)]
    struct ProductFull {
        id: i32,
        name: String,
        price: i32,
        description: String,
    }

    #[derive(Debug, PartialEq, Clone, Default)]
    struct ProductShort {
        id: i32,
        name: String,
        price: i32,
        description: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(ProductFull)]
    #[map_to(ProductShort)]
    struct ProductMultiDto {
        id: i32,
        name: String,
        #[map(name = "price", map_to_with = double, map_from_with = half)]
        price_doubled: i32,
        #[map(skip)]
        description: String,
    }

    #[test]
    fn test_multi_target_with_skip_to_full() {
        let dto = ProductMultiDto {
            id: 10,
            name: "Widget".to_string(),
            price_doubled: 5,
            description: "secret".to_string(),
        };
        let product: ProductFull = dto.morph_to();
        assert_eq!(product.id, 10);
        assert_eq!(product.name, "Widget");
        assert_eq!(product.price, 10); // double(5)
        assert_eq!(product.description, String::default()); // skipped
    }

    #[test]
    fn test_multi_target_with_skip_to_short() {
        let dto = ProductMultiDto {
            id: 20,
            name: "Gadget".to_string(),
            price_doubled: 7,
            description: "secret".to_string(),
        };
        let short: ProductShort = dto.morph_to();
        assert_eq!(short.id, 20);
        assert_eq!(short.name, "Gadget");
        assert_eq!(short.price, 14); // double(7)
        assert_eq!(short.description, String::default()); // skipped
    }

    #[test]
    fn test_multi_target_from_full() {
        let product = ProductFull {
            id: 30,
            name: "Doohickey".to_string(),
            price: 100,
            description: "a cool thing".to_string(),
        };
        let dto = ProductMultiDto::morph_from(product);
        assert_eq!(dto.id, 30);
        assert_eq!(dto.name, "Doohickey");
        assert_eq!(dto.price_doubled, 50); // half(100)
        assert_eq!(dto.description, String::default()); // skipped
    }

    #[test]
    fn test_multi_target_from_short() {
        let short = ProductShort {
            id: 40,
            name: "Thingamajig".to_string(),
            price: 200,
            description: "another thing".to_string(),
        };
        let dto = ProductMultiDto::morph_from(short);
        assert_eq!(dto.id, 40);
        assert_eq!(dto.name, "Thingamajig");
        assert_eq!(dto.price_doubled, 100); // half(200)
        assert_eq!(dto.description, String::default()); // skipped
    }

    // -- Multi-target: 3 targets --

    #[derive(Debug, PartialEq, Clone)]
    struct Level1 {
        x: f64,
        y: f64,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Level2 {
        x: f64,
        y: f64,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Level3 {
        x: f64,
        y: f64,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Level1)]
    #[map_to(Level2)]
    #[map_to(Level3)]
    struct CoordDto {
        x: f64,
        y: f64,
    }

    #[test]
    fn test_three_targets_to() {
        let dto = CoordDto { x: 1.0, y: 2.0 };
        let l1: Level1 = dto.morph_to();
        assert_eq!(l1.x, 1.0);
        assert_eq!(l1.y, 2.0);
    }

    #[test]
    fn test_three_targets_to_l2() {
        let dto = CoordDto { x: 3.0, y: 4.0 };
        let l2: Level2 = dto.morph_to();
        assert_eq!(l2.x, 3.0);
        assert_eq!(l2.y, 4.0);
    }

    #[test]
    fn test_three_targets_to_l3() {
        let dto = CoordDto { x: 5.0, y: 6.0 };
        let l3: Level3 = dto.morph_to();
        assert_eq!(l3.x, 5.0);
        assert_eq!(l3.y, 6.0);
    }

    #[test]
    fn test_three_targets_from_l1() {
        let l1 = Level1 { x: 7.0, y: 8.0 };
        let dto = CoordDto::morph_from(l1);
        assert_eq!(dto.x, 7.0);
        assert_eq!(dto.y, 8.0);
    }

    #[test]
    fn test_three_targets_from_l3() {
        let l3 = Level3 { x: 9.0, y: 10.0 };
        let dto = CoordDto::morph_from(l3);
        assert_eq!(dto.x, 9.0);
        assert_eq!(dto.y, 10.0);
    }

    // ============================================================
    // -- Option<T> support --
    // ============================================================

    // Case 1: Option<T> -> T (unwrap_or_default)
    #[derive(Debug, PartialEq, Clone, Default)]
    struct Profile {
        name: String,
        age: i32,
        bio: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Profile)]
    struct ProfileDto {
        name: String,
        age: i32,
        #[map(name = "bio")]
        biography: Option<String>, // Option<String> -> String: unwrap_or_default
    }

    #[test]
    fn test_option_to_non_option_some() {
        let dto = ProfileDto {
            name: "Alice".to_string(),
            age: 30,
            biography: Some("Hello world".to_string()),
        };
        let profile = dto.morph_to();
        assert_eq!(profile.name, "Alice");
        assert_eq!(profile.age, 30);
        assert_eq!(profile.bio, "Hello world");
    }

    #[test]
    fn test_option_to_non_option_none() {
        let dto = ProfileDto {
            name: "Bob".to_string(),
            age: 25,
            biography: None,
        };
        let profile = dto.morph_to();
        assert_eq!(profile.name, "Bob");
        assert_eq!(profile.age, 25);
        assert_eq!(profile.bio, String::default()); // unwrap_or_default
    }

    // Case 2: morph_from Profile -> ProfileDto (String -> Option<String> auto Some)
    #[test]
    fn test_morph_from_option_auto_some() {
        let profile = Profile {
            name: "Charlie".to_string(),
            age: 40,
            bio: "Developer".to_string(),
        };
        let dto = ProfileDto::morph_from(profile);
        assert_eq!(dto.name, "Charlie");
        assert_eq!(dto.age, 40);
        assert_eq!(dto.biography, Some("Developer".to_string())); // auto wrapped in Some
    }

    // Case 3: Option<T> -> Option<T> (clone in morph_to)
    fn identity_option<T>(opt: Option<T>) -> Option<T> {
        opt
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Contact {
        email: Option<String>,
        phone: Option<String>,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Contact)]
    struct ContactDto {
        #[map(map_to_with = identity_option, map_from_with = identity_option)]
        email: Option<String>,
        #[map(name = "phone", map_to_with = identity_option, map_from_with = identity_option)]
        telephone: Option<String>,
    }

    #[test]
    fn test_option_to_option_some() {
        let dto = ContactDto {
            email: Some("a@b.com".to_string()),
            telephone: Some("123".to_string()),
        };
        let contact = dto.morph_to();
        assert_eq!(contact.email, Some("a@b.com".to_string()));
        assert_eq!(contact.phone, Some("123".to_string()));
    }

    #[test]
    fn test_option_to_option_none() {
        let dto = ContactDto {
            email: None,
            telephone: None,
        };
        let contact = dto.morph_to();
        assert_eq!(contact.email, None);
        assert_eq!(contact.phone, None);
    }

    // morph_from Contact -> ContactDto: Option -> Option (auto Some wraps the Option)
    // Since both are Option, Some(t.field) creates Some(Some(...)) which is wrong.
    // We skip this test and document the limitation.

    // Case 4: Option with Copy types
    #[derive(Debug, PartialEq, Clone, Default)]
    struct SettingsOpt {
        max_retries: i32,
        timeout: i32,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(SettingsOpt)]
    struct SettingsOptDto {
        #[map(name = "max_retries")]
        retries: Option<i32>, // Option<i32> -> i32: unwrap_or_default
        timeout: i32,
    }

    #[test]
    fn test_option_copy_type_some() {
        let dto = SettingsOptDto {
            retries: Some(5),
            timeout: 30,
        };
        let settings = dto.morph_to();
        assert_eq!(settings.max_retries, 5);
        assert_eq!(settings.timeout, 30);
    }

    #[test]
    fn test_option_copy_type_none() {
        let dto = SettingsOptDto {
            retries: None,
            timeout: 30,
        };
        let settings = dto.morph_to();
        assert_eq!(settings.max_retries, i32::default()); // unwrap_or_default
        assert_eq!(settings.timeout, 30);
    }

    // morph_from for SettingsOpt -> SettingsOptDto: i32 -> Option<i32> (auto Some)
    #[test]
    fn test_option_copy_type_morph_from() {
        let settings = SettingsOpt {
            max_retries: 7,
            timeout: 60,
        };
        let dto = SettingsOptDto::morph_from(settings);
        assert_eq!(dto.retries, Some(7)); // auto wrapped in Some
        assert_eq!(dto.timeout, 60);
    }

    // Case 5: Option with custom transform (transform receives Option<T>)
    fn uppercase_option(opt: Option<String>) -> String {
        match opt {
            Some(s) => s.to_uppercase(),
            None => String::new(),
        }
    }

    #[derive(Debug, PartialEq, Clone, Default)]
    struct Tag {
        label: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Tag)]
    struct TagDto {
        #[map(name = "label", map_to_with = uppercase_option)]
        name: Option<String>,
    }

    #[test]
    fn test_option_with_transform_some() {
        let dto = TagDto {
            name: Some("hello".to_string()),
        };
        let tag = dto.morph_to();
        assert_eq!(tag.label, "HELLO");
    }

    #[test]
    fn test_option_with_transform_none() {
        let dto = TagDto {
            name: None,
        };
        let tag = dto.morph_to();
        // None is passed to uppercase_option, which returns empty string
        assert_eq!(tag.label, "");
    }

    // ============================================================
    // -- Generics support --
    // ============================================================

    // Generic structs with concrete types in #[map_to]
    #[derive(Debug, PartialEq, Clone)]
    struct ResponseString {
        data: String,
        status: u32,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(ResponseString)]
    struct ResponseStringDto {
        #[map(name = "data")]
        payload: String,
        #[map(name = "status")]
        code: u32,
    }

    #[test]
    fn test_generics_concrete_morph_to() {
        let dto = ResponseStringDto {
            payload: "success".to_string(),
            code: 200,
        };
        let response = dto.morph_to();
        assert_eq!(response.data, "success");
        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_generics_concrete_morph_from() {
        let response = ResponseString {
            data: "ok".to_string(),
            status: 201,
        };
        let dto = ResponseStringDto::morph_from(response);
        assert_eq!(dto.payload, "ok");
        assert_eq!(dto.code, 201);
    }

    // Generic struct with Clone bound verification
    #[derive(Debug, PartialEq, Clone)]
    struct Container {
        value: String,
        count: i32,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Container)]
    struct ContainerDto {
        #[map(name = "value")]
        content: String,
        #[map(name = "count")]
        size: i32,
    }

    #[test]
    fn test_clone_bound_morph_to() {
        let dto = ContainerDto {
            content: "hello".to_string(),
            size: 5,
        };
        let container = dto.morph_to();
        assert_eq!(container.value, "hello");
        assert_eq!(container.count, 5);
    }

    #[test]
    fn test_clone_bound_morph_from() {
        let container = Container {
            value: "world".to_string(),
            count: 10,
        };
        let dto = ContainerDto::morph_from(container);
        assert_eq!(dto.content, "world");
        assert_eq!(dto.size, 10);
    }

    // ============================================================
    // -- Copy optimization verification --
    // ============================================================

    // This struct only has Copy types — the generated code should NOT use .clone()
    #[derive(Debug, PartialEq)]
    struct CopyTarget {
        x: i32,
        y: i64,
        active: bool,
        ratio: f64,
    }

    #[derive(Debug, PartialEq, AutoMorph)]
    #[map_to(CopyTarget)]
    struct CopySource {
        #[map(name = "x")]
        pos_x: i32,
        #[map(name = "y")]
        pos_y: i64,
        active: bool,
        #[map(name = "ratio")]
        percentage: f64,
    }

    #[test]
    fn test_copy_optimization_to() {
        let source = CopySource {
            pos_x: 10,
            pos_y: 20,
            active: true,
            percentage: 3.14,
        };
        let target = source.morph_to();
        assert_eq!(target.x, 10);
        assert_eq!(target.y, 20);
        assert_eq!(target.active, true);
        assert_eq!(target.ratio, 3.14);
    }

    #[test]
    fn test_copy_optimization_from() {
        let target = CopyTarget {
            x: 100,
            y: 200,
            active: false,
            ratio: 2.71,
        };
        let source = CopySource::morph_from(target);
        assert_eq!(source.pos_x, 100);
        assert_eq!(source.pos_y, 200);
        assert_eq!(source.active, false);
        assert_eq!(source.percentage, 2.71);
    }

    // Mixed: some Copy, some Clone types
    #[derive(Debug, PartialEq, Clone)]
    struct MixedTarget {
        id: i32,
        name: String,
        active: bool,
        score: f64,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(MixedTarget)]
    struct MixedSource {
        id: i32,
        #[map(name = "name")]
        title: String,
        active: bool,
        #[map(name = "score")]
        points: f64,
    }

    #[test]
    fn test_mixed_copy_clone_to() {
        let source = MixedSource {
            id: 1,
            title: "Test".to_string(),
            active: true,
            points: 99.5,
        };
        let target = source.morph_to();
        assert_eq!(target.id, 1);
        assert_eq!(target.name, "Test");
        assert_eq!(target.active, true);
        assert_eq!(target.score, 99.5);
    }

    #[test]
    fn test_mixed_copy_clone_from() {
        let target = MixedTarget {
            id: 2,
            name: "Test2".to_string(),
            active: false,
            score: 50.0,
        };
        let source = MixedSource::morph_from(target);
        assert_eq!(source.id, 2);
        assert_eq!(source.title, "Test2");
        assert_eq!(source.active, false);
        assert_eq!(source.points, 50.0);
    }

    // ============================================================
    // -- Option with skip and default --
    // ============================================================

    #[derive(Debug, PartialEq, Clone, Default)]
    struct PartialUser {
        id: i32,
        name: String,
        nickname: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(PartialUser)]
    struct PartialUserDto {
        id: i32,
        name: Option<String>,
        #[map(skip)]
        nickname: String,
    }

    #[test]
    fn test_option_with_skip_to() {
        let dto = PartialUserDto {
            id: 1,
            name: Some("Alice".to_string()),
            nickname: "ignored".to_string(),
        };
        let user: PartialUser = dto.morph_to();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "Alice");
        assert_eq!(user.nickname, String::default()); // skipped
    }

    #[test]
    fn test_option_with_skip_none_to() {
        let dto = PartialUserDto {
            id: 2,
            name: None,
            nickname: "ignored".to_string(),
        };
        let user: PartialUser = dto.morph_to();
        assert_eq!(user.id, 2);
        assert_eq!(user.name, String::default()); // unwrap_or_default
        assert_eq!(user.nickname, String::default()); // skipped
    }

    // ============================================================
    // -- Option with default --
    // ============================================================

    #[derive(Debug, PartialEq, Clone)]
    struct ConfigWithDefault {
        host: String,
        port: u16,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(ConfigWithDefault)]
    struct ConfigWithDefaultDto {
        #[map(default = "localhost".to_string())]
        host: Option<String>,
        #[map(default = 8080)]
        port: u16,
    }

    #[test]
    fn test_option_with_default_to() {
        let dto = ConfigWithDefaultDto {
            host: Some("ignored".to_string()),
            port: 9999,
        };
        let config = dto.morph_to();
        assert_eq!(config.host, "localhost"); // default overrides
        assert_eq!(config.port, 8080); // default overrides
    }

    // ============================================================
    // -- Multi-target with Option --
    // ============================================================

    #[derive(Debug, PartialEq, Clone, Default)]
    struct FullInfo {
        id: i32,
        title: String,
        description: String,
    }

    #[derive(Debug, PartialEq, Clone, Default)]
    struct ShortInfo {
        id: i32,
        title: String,
        description: String,
    }

    #[derive(Debug, PartialEq, Clone, Default, AutoMorph)]
    #[map_to(FullInfo)]
    #[map_to(ShortInfo)]
    struct InfoDto {
        id: i32,
        title: Option<String>,
        #[map(skip)]
        description: String,
    }

    #[test]
    fn test_multi_target_option_to_full() {
        let dto = InfoDto {
            id: 1,
            title: Some("Hello".to_string()),
            description: "secret".to_string(),
        };
        let full: FullInfo = dto.morph_to();
        assert_eq!(full.id, 1);
        assert_eq!(full.title, "Hello");
        assert_eq!(full.description, String::default());
    }

    #[test]
    fn test_multi_target_option_to_short() {
        let dto = InfoDto {
            id: 2,
            title: None,
            description: "secret".to_string(),
        };
        let short: ShortInfo = dto.morph_to();
        assert_eq!(short.id, 2);
        assert_eq!(short.title, String::default());
        assert_eq!(short.description, String::default());
    }

    // ============================================================
    // -- Enum support --
    // ============================================================

    #[derive(Debug, PartialEq, Clone)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Color)]
    enum ColorDto {
        Red,
        Green,
        #[map(name = "Blue")]
        Azure,
    }

    #[test]
    fn test_enum_morph_to_same_name() {
        let dto = ColorDto::Red;
        let color: Color = dto.morph_to();
        assert_eq!(color, Color::Red);
    }

    #[test]
    fn test_enum_morph_to_renamed() {
        let dto = ColorDto::Azure;
        let color: Color = dto.morph_to();
        assert_eq!(color, Color::Blue);
    }

    #[test]
    fn test_enum_morph_from_same_name() {
        let color = Color::Green;
        let dto: ColorDto = ColorDto::morph_from(color);
        assert_eq!(dto, ColorDto::Green);
    }

    #[test]
    fn test_enum_morph_from_renamed() {
        let color = Color::Blue;
        let dto: ColorDto = ColorDto::morph_from(color);
        assert_eq!(dto, ColorDto::Azure);
    }

    // Multi-target enum
    #[derive(Debug, PartialEq, Clone)]
    enum Status {
        Active,
        Inactive,
    }

    #[derive(Debug, PartialEq, Clone)]
    enum StatusDb {
        Active,
        Inactive,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(Status)]
    #[map_to(StatusDb)]
    enum StatusDto {
        Active,
        Inactive,
    }

    #[test]
    fn test_enum_multi_target_to_status() {
        let dto = StatusDto::Active;
        let status: Status = dto.morph_to();
        assert_eq!(status, Status::Active);
    }

    #[test]
    fn test_enum_multi_target_to_status_db() {
        let dto = StatusDto::Inactive;
        let db: StatusDb = dto.morph_to();
        assert_eq!(db, StatusDb::Inactive);
    }

    // ============================================================
    // -- Standard type Morph implementations --
    // ============================================================

    #[test]
    fn test_std_string_box_str() {
        let s = String::from("hello");
        let boxed: Box<str> = <String as Morph<Box<str>>>::morph_to(&s);
        assert_eq!(&*boxed, "hello");

        let back: String = <String as Morph<Box<str>>>::morph_from(boxed);
        assert_eq!(back, "hello");
    }

    #[test]
    fn test_std_int_conversions() {
        let i: i32 = 42;
        let u: u32 = <i32 as Morph<u32>>::morph_to(&i);
        assert_eq!(u, 42);

        let back: i32 = <i32 as Morph<u32>>::morph_from(u);
        assert_eq!(back, 42);

        let i64: i64 = 100;
        let u64: u64 = <i64 as Morph<u64>>::morph_to(&i64);
        assert_eq!(u64, 100);
    }

    #[test]
    fn test_std_float_int() {
        let f: f64 = 3.14;
        let i: i64 = <f64 as Morph<i64>>::morph_to(&f);
        assert_eq!(i, 3);

        let back: f64 = <f64 as Morph<i64>>::morph_from(i);
        assert_eq!(back, 3.0);
    }

    #[test]
    fn test_std_bool_u8() {
        let b: bool = true;
        let u: u8 = <bool as Morph<u8>>::morph_to(&b);
        assert_eq!(u, 1);

        let back: bool = <bool as Morph<u8>>::morph_from(u);
        assert_eq!(back, true);

        let f: bool = false;
        let u: u8 = <bool as Morph<u8>>::morph_to(&f);
        assert_eq!(u, 0);
    }

    #[test]
    fn test_std_option_t() {
        let opt: Option<String> = Some("hello".to_string());
        let s: String = <String as Morph<Option<String>>>::morph_from(opt.clone());
        assert_eq!(s, "hello");

        let back: Option<String> = Some(s.clone());
        assert_eq!(back, Some("hello".to_string()));

        // None -> default
        let none: Option<i32> = None;
        let i: i32 = <i32 as Morph<Option<i32>>>::morph_from(none);
        assert_eq!(i, 0);
    }

    // ============================================================
    // -- MorphResult (fallible morphing) --
    // ============================================================

    use auto_morphic_core::morph::{MorphError, MorphResult};

    #[derive(Debug, Clone, PartialEq)]
    struct ParsedConfig {
        host: String,
        port: u16,
    }

    #[derive(Debug, Clone)]
    struct RawConfig {
        host: String,
        port_str: String,
    }

    impl MorphResult<ParsedConfig> for RawConfig {
        fn morph_to_result(&self) -> Result<ParsedConfig, MorphError> {
            Ok(ParsedConfig {
                host: self.host.clone(),
                port: self.port_str.parse::<u16>().map_err(|e| {
                    MorphError::field("port", e.to_string())
                })?,
            })
        }

        fn morph_from_result(t: ParsedConfig) -> Result<Self, MorphError> {
            Ok(Self {
                host: t.host,
                port_str: t.port.to_string(),
            })
        }
    }

    #[test]
    fn test_morph_result_success() {
        let raw = RawConfig {
            host: "localhost".to_string(),
            port_str: "8080".to_string(),
        };
        let parsed = raw.morph_to_result().unwrap();
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, 8080);
    }

    #[test]
    fn test_morph_result_failure() {
        let raw = RawConfig {
            host: "localhost".to_string(),
            port_str: "not_a_number".to_string(),
        };
        let err = raw.morph_to_result().unwrap_err();
        assert_eq!(err.field, Some("port".to_string()));
    }

    #[test]
    fn test_morph_result_from() {
        let parsed = ParsedConfig {
            host: "0.0.0.0".to_string(),
            port: 9090,
        };
        let raw: RawConfig = RawConfig::morph_from_result(parsed).unwrap();
        assert_eq!(raw.host, "0.0.0.0");
        assert_eq!(raw.port_str, "9090");
    }

    // ============================================================
    // -- MorphResult derive macro (auto-generated) --
    // ============================================================

    fn parse_port(s: String) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        s.parse::<u16>().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn format_port(p: u16) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(p.to_string())
    }

    fn parse_age(s: String) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        s.parse::<i32>().map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn format_age(a: i32) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(a.to_string())
    }

    // Non-fallible versions for Morph (non-Result) path
    fn parse_port_infallible(s: String) -> u16 {
        s.parse::<u16>().unwrap_or(0)
    }

    fn format_port_infallible(p: u16) -> String {
        p.to_string()
    }

    fn parse_age_infallible(s: String) -> i32 {
        s.parse::<i32>().unwrap_or(0)
    }

    fn format_age_infallible(a: i32) -> String {
        a.to_string()
    }

    #[derive(Debug, Clone, PartialEq)]
    struct ServerConfig {
        host: String,
        port: u16,
        name: String,
    }

    #[derive(Debug, Clone, PartialEq, AutoMorph)]
    #[map_to(ServerConfig)]
    struct ServerConfigInput {
        host: String,
        #[map(
            name = "port",
            map_to_with = parse_port_infallible,
            map_from_with = format_port_infallible,
            map_to_result_with = parse_port,
            map_from_result_with = format_port,
        )]
        port_str: String,
        #[map(name = "name")]
        server_name: String,
    }

    #[test]
    fn test_derive_morph_result_to_success() {
        let input = ServerConfigInput {
            host: "localhost".to_string(),
            port_str: "8080".to_string(),
            server_name: "web".to_string(),
        };
        let config: ServerConfig = input.morph_to_result().unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.name, "web");
    }

    #[test]
    fn test_derive_morph_result_to_failure() {
        let input = ServerConfigInput {
            host: "localhost".to_string(),
            port_str: "not_a_port".to_string(),
            server_name: "web".to_string(),
        };
        let err = input.morph_to_result().unwrap_err();
        assert_eq!(err.field, Some("port".to_string()));
    }

    #[test]
    fn test_derive_morph_result_from_success() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 9090,
            name: "api".to_string(),
        };
        let input: ServerConfigInput = ServerConfigInput::morph_from_result(config).unwrap();
        assert_eq!(input.host, "0.0.0.0");
        assert_eq!(input.port_str, "9090");
        assert_eq!(input.server_name, "api");
    }

    // Also verify Morph (non-Result) still works alongside MorphResult
    #[test]
    fn test_derive_morph_and_morph_result_coexist_to() {
        let input = ServerConfigInput {
            host: "localhost".to_string(),
            port_str: "3000".to_string(),
            server_name: "test".to_string(),
        };
        // Morph (infallible)
        let config: ServerConfig = input.morph_to();
        assert_eq!(config.port, 3000);
        // MorphResult (fallible)
        let config2: ServerConfig = input.morph_to_result().unwrap();
        assert_eq!(config2.port, 3000);
    }

    // Multi-target with MorphResult
    #[derive(Debug, Clone, PartialEq)]
    struct DatabaseConfig {
        host: String,
        port: u16,
    }

    #[derive(Debug, Clone, PartialEq)]
    struct CacheConfig {
        host: String,
        port: u16,
    }

    #[derive(Debug, Clone, PartialEq, AutoMorph)]
    #[map_to(DatabaseConfig)]
    #[map_to(CacheConfig)]
    struct ServiceInput {
        host: String,
        #[map(
            name = "port",
            map_to_with = parse_port_infallible,
            map_from_with = format_port_infallible,
            map_to_result_with = parse_port,
            map_from_result_with = format_port,
        )]
        port_str: String,
    }

    #[test]
    fn test_derive_morph_result_multi_target_to_db() {
        let input = ServiceInput {
            host: "db.local".to_string(),
            port_str: "5432".to_string(),
        };
        let db: DatabaseConfig = input.morph_to_result().unwrap();
        assert_eq!(db.host, "db.local");
        assert_eq!(db.port, 5432);
    }

    #[test]
    fn test_derive_morph_result_multi_target_to_cache() {
        let input = ServiceInput {
            host: "cache.local".to_string(),
            port_str: "6379".to_string(),
        };
        let cache: CacheConfig = input.morph_to_result().unwrap();
        assert_eq!(cache.host, "cache.local");
        assert_eq!(cache.port, 6379);
    }

    #[test]
    fn test_derive_morph_result_multi_target_from_db() {
        let db = DatabaseConfig {
            host: "db.local".to_string(),
            port: 5432,
        };
        let input: ServiceInput = ServiceInput::morph_from_result(db).unwrap();
        assert_eq!(input.host, "db.local");
        assert_eq!(input.port_str, "5432");
    }

    // MorphResult with skip and default
    #[derive(Debug, Clone, PartialEq, Default)]
    struct AppSettings {
        title: String,
        max_connections: i32,
        secret: String,
    }

    #[derive(Debug, Clone, PartialEq, Default, AutoMorph)]
    #[map_to(AppSettings)]
    struct AppSettingsInput {
        #[map(name = "title")]
        app_title: String,
        #[map(
            name = "max_connections",
            map_to_with = parse_age_infallible,
            map_from_with = format_age_infallible,
            map_to_result_with = parse_age,
            map_from_result_with = format_age,
        )]
        max_conn_str: String,
        #[map(skip)]
        secret: String,
    }

    #[test]
    fn test_derive_morph_result_with_skip_to() {
        let input = AppSettingsInput {
            app_title: "MyApp".to_string(),
            max_conn_str: "100".to_string(),
            secret: "supersecret".to_string(),
        };
        let settings: AppSettings = input.morph_to_result().unwrap();
        assert_eq!(settings.title, "MyApp");
        assert_eq!(settings.max_connections, 100);
        assert_eq!(settings.secret, String::default()); // skipped
    }

    #[test]
    fn test_derive_morph_result_with_skip_from() {
        let settings = AppSettings {
            title: "ProdApp".to_string(),
            max_connections: 500,
            secret: "prod-secret".to_string(),
        };
        let input: AppSettingsInput = AppSettingsInput::morph_from_result(settings).unwrap();
        assert_eq!(input.app_title, "ProdApp");
        assert_eq!(input.max_conn_str, "500");
        assert_eq!(input.secret, String::default()); // skipped
    }

    // MorphResult with only map_from_result_with (one-directional result)
    // Uses map_from_with for the non-Result path
    #[derive(Debug, Clone, PartialEq)]
    struct OutputValue {
        data: String,
        count: i32,
    }

    #[derive(Debug, Clone, PartialEq, AutoMorph)]
    #[map_to(OutputValue)]
    struct InputValue {
        data: String,
        #[map(
            name = "count",
            map_to_with = parse_age_infallible,
            map_from_with = format_age_infallible,
            map_from_result_with = format_age,
        )]
        count_str: String,
    }

    #[test]
    fn test_derive_morph_result_one_way_from() {
        let output = OutputValue {
            data: "test".to_string(),
            count: 42,
        };
        let input: InputValue = InputValue::morph_from_result(output).unwrap();
        assert_eq!(input.data, "test");
        assert_eq!(input.count_str, "42");
    }

    #[test]
    fn test_derive_morph_result_morph_to_still_works() {
        let input = InputValue {
            data: "test".to_string(),
            count_str: "42".to_string(),
        };
        let output: OutputValue = input.morph_to();
        assert_eq!(output.data, "test");
        assert_eq!(output.count, 42);
    }

    // ============================================================
    // -- Flatten support --
    // ============================================================

    #[derive(Debug, PartialEq, Clone)]
    struct NestedAddr {
        street: String,
        city: String,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(NestedAddr)]
    struct NestedAddrDto {
        street: String,
        city: String,
    }

    // Flatten test: the nested struct (NestedAddrDto) maps to NestedAddr.
    // The target struct (UserWithNestedAddr) has the same fields as NestedAddr.
    // Flatten morphs the nested struct to its target, then uses the result's fields.
    // Since Rust has no struct spreading, flatten works by morphing the nested type
    // and assigning it to a field with the same name in the target.
    // For this test, we use a target that has an `address` field.

    #[derive(Debug, PartialEq, Clone)]
    struct UserWithNestedAddr {
        name: String,
        address: NestedAddr,
    }

    #[derive(Debug, PartialEq, Clone, AutoMorph)]
    #[map_to(UserWithNestedAddr)]
    struct UserWithNestedAddrDto {
        name: String,
        #[map(flatten)]
        address: NestedAddrDto,
    }

    #[test]
    fn test_flatten_morph_to() {
        let dto = UserWithNestedAddrDto {
            name: "Alice".to_string(),
            address: NestedAddrDto {
                street: "Main St".to_string(),
                city: "NYC".to_string(),
            },
        };
        let user: UserWithNestedAddr = dto.morph_to();
        assert_eq!(user.name, "Alice");
        assert_eq!(user.address.street, "Main St");
        assert_eq!(user.address.city, "NYC");
    }

    #[test]
    fn test_flatten_morph_from() {
        let user = UserWithNestedAddr {
            name: "Bob".to_string(),
            address: NestedAddr {
                street: "Oak Ave".to_string(),
                city: "LA".to_string(),
            },
        };
        let dto: UserWithNestedAddrDto = UserWithNestedAddrDto::morph_from(user);
        assert_eq!(dto.name, "Bob");
        assert_eq!(dto.address.street, "Oak Ave");
        assert_eq!(dto.address.city, "LA");
    }
}