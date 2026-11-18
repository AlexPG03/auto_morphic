//! AutoMorphic — Ejemplo completo de todas las features
//!
//! Este ejemplo demuestra cada funcionalidad de la macro `#[derive(AutoMorph)]`:
//!
//! 1. Mapeo básico con renombrado de campos
//! 2. Transformaciones personalizadas (map_to_with / map_from_with)
//! 3. Saltar campos (skip) y valores por defecto (default)
//! 4. Conversión automática Option<T> ↔ T
//! 5. Mapeo de enums con unit variants
//! 6. Multi-target: un DTO a varios targets
//! 7. Mapeo fallible con MorphResult
//! 8. Conversión de tipos estándar (i32 ↔ u32, bool ↔ u8, etc.)
//! 9. Flatten de structs anidados
//!
//! Ejecuta con: `cargo run --example app`

use auto_morphic::{Morph, MorphResult, MorphError, AutoMorph};

// ============================================================================
// 1. Mapeo básico con renombrado de campos
// ============================================================================
//
// El caso más común: dos structs con los mismos datos pero nombres distintos.
// `#[map(name = "...")]` indica el nombre del campo en el struct destino.

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: i32,
    username: String,
    email: String,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(User)]
struct UserDto {
    id: i32,
    #[map(name = "username")]
    name: String,
    #[map(name = "email")]
    email_address: String,
}

// ============================================================================
// 2. Transformaciones personalizadas
// ============================================================================
//
// `map_to_with` y `map_from_with` aceptan rutas a funciones que transforman
// el valor durante la conversión. La función recibe el valor del campo origen
// y retorna el tipo del campo destino.

fn uppercase(s: String) -> String {
    s.to_uppercase()
}

fn lowercase(s: String) -> String {
    s.to_lowercase()
}

#[derive(Debug, Clone, PartialEq)]
struct Product {
    id: i32,
    name: String,
    price_cents: i32,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Product)]
struct ProductDto {
    id: i32,
    #[map(map_to_with = uppercase, map_from_with = lowercase)]
    name: String,
    #[map(name = "price_cents")]
    price_dollars: i32,
}

// ============================================================================
// 3. Skip y valores por defecto
// ============================================================================
//
// `#[map(skip)]` omite un campo en ambas direcciones (requiere Default).
// `#[map(default = expr)]` usa una expresión como valor por defecto.
// `#[map(default)]` usa `Default::default()` en ambas direcciones.

#[derive(Debug, Clone, PartialEq, Default)]
struct SecureUser {
    id: i32,
    username: String,
    password_hash: String,
    role: String,
}

#[derive(Debug, Clone, PartialEq, Default, AutoMorph)]
#[map_to(SecureUser)]
struct SecureUserDto {
    id: i32,
    username: String,
    #[map(skip)]
    password_hash: String,
    #[map(default = "user".to_string())]
    role: String,
}

// ============================================================================
// 4. Soporte para Option<T>
// ============================================================================
//
// AutoMorph detecta automáticamente cuando un campo es Option<T> y el destino
// es T (o viceversa):
//   - Option<T> → T: usa unwrap_or_default()
//   - T → Option<T>: envuelve en Some()

#[derive(Debug, Clone, PartialEq, Default)]
struct Profile {
    name: String,
    bio: String,
    age: i32,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Profile)]
struct ProfileDto {
    name: String,
    #[map(name = "bio")]
    biography: Option<String>,
    age: i32,
}

// ============================================================================
// 5. Soporte para enums (unit variants)
// ============================================================================
//
// Los enums con unit variants se mapean por nombre.
// Usa `#[map(name = "Variant")]` para renombrar una variante.

#[derive(Debug, Clone, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Status)]
enum StatusDto {
    Active,
    Inactive,
    #[map(name = "Pending")]
    AwaitingApproval,
}

// ============================================================================
// 6. Multi-target: un DTO a varios targets
// ============================================================================
//
// Múltiples `#[map_to(T)]` generan un `impl Morph<T>` independiente por target.
// Los `#[map(...)]` en los campos se aplican a TODOS los targets.

#[derive(Debug, Clone, PartialEq)]
struct ArticleFull {
    id: i32,
    title: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ArticleSummary {
    id: i32,
    title: String,
    body: String,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(ArticleFull)]
#[map_to(ArticleSummary)]
struct ArticleDto {
    id: i32,
    #[map(name = "title")]
    headline: String,
    #[map(name = "body")]
    content: String,
}

// ============================================================================
// 7. Mapeo fallible con MorphResult
// ============================================================================
//
// Cuando una transformación puede fallar (parsing, validación), usa MorphResult.
// La macro genera `impl MorphResult<T>` automáticamente cuando detecta
// `map_to_result_with` o `map_from_result_with`.

#[derive(Debug, Clone, PartialEq)]
struct Config {
    host: String,
    port: u16,
}

// Manual MorphResult implementation for parsing
impl MorphResult<Config> for ConfigInput {
    fn morph_to_result(&self) -> Result<Config, MorphError> {
        Ok(Config {
            host: self.host.clone(),
            port: self.port_str.parse::<u16>().map_err(|e: std::num::ParseIntError| {
                MorphError::field("port", e.to_string())
            })?,
        })
    }

    fn morph_from_result(t: Config) -> Result<Self, MorphError> {
        Ok(Self {
            host: t.host,
            port_str: t.port.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
struct ConfigInput {
    host: String,
    port_str: String,
}

// ============================================================================
// 8. Flatten de structs anidados
// ============================================================================
//
// `#[map(flatten)]` morpha un struct anidado a su tipo destino usando su propio
// `Morph` y asigna el resultado al campo con el mismo nombre en el target.

#[derive(Debug, Clone, PartialEq)]
struct Address {
    street: String,
    city: String,
}

#[derive(Debug, Clone, PartialEq)]
struct AddressDto {
    street: String,
    city: String,
}

// Address morpha a AddressDto (y viceversa)
impl Morph<AddressDto> for Address {
    fn morph_to(&self) -> AddressDto {
        AddressDto {
            street: self.street.clone(),
            city: self.city.clone(),
        }
    }
    fn morph_from(t: AddressDto) -> Self {
        Self {
            street: t.street,
            city: t.city,
        }
    }
}

impl Morph<Address> for AddressDto {
    fn morph_to(&self) -> Address {
        Address {
            street: self.street.clone(),
            city: self.city.clone(),
        }
    }
    fn morph_from(t: Address) -> Self {
        Self {
            street: t.street,
            city: t.city,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Company {
    name: String,
    address: Address,
}

#[derive(Debug, Clone, PartialEq)]
struct CompanyDto {
    name: String,
    address: AddressDto,
}

#[derive(Debug, Clone, PartialEq, AutoMorph)]
#[map_to(Company)]
struct CompanyInput {
    name: String,
    #[map(flatten)]
    address: AddressDto,
}

// ============================================================================
// 9. Tipos estándar pre-implementados
// ============================================================================
//
// Morph está implementado para pares comunes de tipos:
//   - Enteros: i32 ↔ u32, i64 ↔ u64, i32 ↔ i64, etc.
//   - Floats: f32 ↔ f64, f64 ↔ i64
//   - Bool ↔ u8
//   - String ↔ Box<str>
//   - Option<T> ↔ T (donde T: Default + Clone)

fn demonstrate_standard_types() {
    // String ↔ Box<str>
    let s = String::from("hello");
    let boxed: Box<str> = <String as Morph<Box<str>>>::morph_to(&s);
    println!("  String -> Box<str>: {}", boxed);

    let back: String = <String as Morph<Box<str>>>::morph_from(boxed);
    println!("  Box<str> -> String: {}", back);

    // Enteros: i32 ↔ u32
    let i: i32 = 42;
    let u: u32 = <i32 as Morph<u32>>::morph_to(&i);
    println!("  i32 -> u32: {}", u);

    let back: i32 = <i32 as Morph<u32>>::morph_from(u);
    println!("  u32 -> i32: {}", back);

    // Floats: f64 ↔ i64
    let f: f64 = 3.14;
    let i: i64 = <f64 as Morph<i64>>::morph_to(&f);
    println!("  f64 -> i64: {}", i);

    // Bool ↔ u8
    let b: bool = true;
    let u8_val: u8 = <bool as Morph<u8>>::morph_to(&b);
    println!("  bool -> u8: {}", u8_val);

    let back: bool = <bool as Morph<u8>>::morph_from(u8_val);
    println!("  u8 -> bool: {}", back);

    // Option<T> ↔ T
    let opt: Option<i32> = Some(42);
    let val: i32 = <Option<i32> as Morph<i32>>::morph_to(&opt);
    println!("  Option<i32> -> i32: {}", val);

    let none_opt: Option<i32> = None;
    let default_val: i32 = <Option<i32> as Morph<i32>>::morph_to(&none_opt);
    println!("  None -> i32 (default): {}", default_val);
}

// ============================================================================
// Main — demuestra todas las features
// ============================================================================

fn main() {
    println!("=== AutoMorphic — Ejemplo Completo ===\n");

    // 1. Mapeo básico
    println!("--- 1. Mapeo Básico (renombrado) ---");
    let dto = UserDto {
        id: 1,
        name: "alice".to_string(),
        email_address: "alice@example.com".to_string(),
    };
    let user: User = dto.morph_to();
    println!("  UserDto -> User: {:?}", user);

    let user_back: UserDto = UserDto::morph_from(user);
    println!("  User -> UserDto: {:?}", user_back);
    println!();

    // 2. Transformaciones
    println!("--- 2. Transformaciones Personalizadas ---");
    let product_dto = ProductDto {
        id: 100,
        name: "widget".to_string(),
        price_dollars: 10,
    };
    let product: Product = product_dto.morph_to();
    println!("  ProductDto -> Product: {:?}", product);
    println!("  (nombre convertido a mayúsculas)");

    let product_back: ProductDto = ProductDto::morph_from(product);
    println!("  Product -> ProductDto: {:?}", product_back);
    println!("  (nombre convertido a minúsculas)");
    println!();

    // 3. Skip y Default
    println!("--- 3. Skip y Valores por Defecto ---");
    let secure_dto = SecureUserDto {
        id: 1,
        username: "bob".to_string(),
        password_hash: "secret_hash".to_string(),
        role: "admin".to_string(),
    };
    let secure_user: SecureUser = secure_dto.morph_to();
    println!("  SecureUserDto -> SecureUser: {:?}", secure_user);
    println!("  (password_hash saltado, role con valor por defecto)");
    println!();

    // 4. Option<T>
    println!("--- 4. Option<T> Soporte ---");
    let profile_dto = ProfileDto {
        name: "charlie".to_string(),
        biography: Some("Full-stack developer".to_string()),
        age: 30,
    };
    let profile: Profile = profile_dto.morph_to();
    println!("  ProfileDto (Some bio) -> Profile: {:?}", profile);

    let profile_dto_none = ProfileDto {
        name: "diana".to_string(),
        biography: None,
        age: 25,
    };
    let profile_none: Profile = profile_dto_none.morph_to();
    println!("  ProfileDto (None bio) -> Profile: {:?}", profile_none);
    println!("  (bio vacío por unwrap_or_default)");
    println!();

    // 5. Enums
    println!("--- 5. Mapeo de Enums ---");
    let status_dto = StatusDto::AwaitingApproval;
    let status: Status = status_dto.morph_to();
    println!("  StatusDto::AwaitingApproval -> Status: {:?}", status);

    let status_back: StatusDto = StatusDto::morph_from(Status::Pending);
    println!("  Status::Pending -> StatusDto: {:?}", status_back);
    println!();

    // 6. Multi-target
    println!("--- 6. Multi-Target ---");
    let article_dto = ArticleDto {
        id: 42,
        headline: "Breaking News".to_string(),
        content: "Full story here...".to_string(),
    };
    let full: ArticleFull = article_dto.morph_to();
    println!("  ArticleDto -> ArticleFull: {:?}", full);
    println!("  (mismo DTO puede mapear a ArticleFull o ArticleSummary)");
    println!();

    // 7. MorphResult (mapeo fallible)
    println!("--- 7. Mapeo Fallible (MorphResult) ---");
    let config_input = ConfigInput {
        host: "localhost".to_string(),
        port_str: "8080".to_string(),
    };
    match config_input.morph_to_result() {
        Ok(config) => println!("  ConfigInput -> Config: {:?}", config),
        Err(e) => eprintln!("  Error: {}", e),
    }

    let bad_config = ConfigInput {
        host: "localhost".to_string(),
        port_str: "not_a_number".to_string(),
    };
    match bad_config.morph_to_result() {
        Ok(_) => println!("  Unexpected success"),
        Err(e) => println!("  Error esperado: {}", e),
    }
    println!();

    // 8. Flatten
    println!("--- 8. Flatten (structs anidados) ---");
    let company_input = CompanyInput {
        name: "Acme Corp".to_string(),
        address: AddressDto {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
        },
    };
    let company: Company = company_input.morph_to();
    println!("  CompanyInput -> Company: {:?}", company);
    println!("  (AddressDto aplanado a Address)");
    println!();

    // 9. Tipos estándar
    println!("--- 9. Tipos Estándar ---");
    demonstrate_standard_types();
    println!();

    println!("=== ¡Todas las features demostradas correctamente! ===");
}
