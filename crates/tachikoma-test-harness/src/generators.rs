//! Test data generators for creating realistic test data.

use fake::{Fake, Faker};
use fake::faker::internet::en::*;
use fake::faker::name::en::*;
use fake::faker::lorem::en::*;
use fake::faker::filesystem::en::*;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub use fake;
pub use proptest::prelude::*;

pub mod domain;
pub mod api;

/// Generator context with optional seed for reproducibility
pub struct GeneratorContext {
    rng: StdRng,
}

impl GeneratorContext {
    /// Create a new generator context with random seed
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }

    /// Create a generator context with specific seed for reproducibility
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Get mutable reference to RNG
    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }
}

impl Default for GeneratorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random email address
pub fn email() -> String {
    FreeEmail().fake()
}

/// Generate a random username
pub fn username() -> String {
    Username().fake()
}

/// Generate a random full name
pub fn full_name() -> String {
    Name().fake()
}

/// Generate a random first name
pub fn first_name() -> String {
    FirstName().fake()
}

/// Generate a random last name
pub fn last_name() -> String {
    LastName().fake()
}

/// Generate random words
pub fn words(count: usize) -> Vec<String> {
    Words(count..count + 1).fake()
}

/// Generate a random sentence
pub fn sentence() -> String {
    Sentence(3..10).fake()
}

/// Generate a random paragraph
pub fn paragraph() -> String {
    Paragraph(3..7).fake()
}

/// Generate a random file path
pub fn file_path() -> String {
    FilePath().fake()
}

/// Generate a random file name
pub fn file_name() -> String {
    FileName().fake()
}

/// Generate a random directory path
pub fn dir_path() -> String {
    DirPath().fake()
}

/// Generate a random hex string
pub fn hex_string(length: usize) -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| format!("{:x}", rng.gen::<u8>() % 16))
        .collect()
}

/// Generate a random alphanumeric string
pub fn alphanumeric(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Generate a UUID v4
pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate a timestamp within a range
pub fn timestamp_between(start: i64, end: i64) -> i64 {
    use rand::Rng;
    rand::thread_rng().gen_range(start..end)
}

/// Generate a random boolean with probability
pub fn bool_with_probability(probability: f64) -> bool {
    use rand::Rng;
    rand::thread_rng().gen_bool(probability)
}

// ============================================
// Legacy compatibility functions
// ============================================

/// Generate a random string of specified length
pub fn random_string(length: usize) -> String {
    Word().fake::<String>().chars().cycle().take(length).collect()
}

/// Generate a random alphanumeric string
pub fn random_alphanumeric(length: usize) -> String {
    alphanumeric(length)
}

/// Generate a random email address
pub fn random_email() -> String {
    email()
}

/// Generate a random UUID string
pub fn random_uuid() -> String {
    uuid()
}

/// Generate a random port number (for testing)
pub fn random_port() -> u16 {
    use fake::faker::number::en::*;
    NumberWithFormat("####").fake::<String>()
        .parse::<u16>()
        .unwrap_or(8080)
        .max(1024)
        .max(65535)
}

/// Generate random test data for different types
pub struct TestDataGenerator;

impl TestDataGenerator {
    /// Generate a vector of random strings
    pub fn strings(count: usize, length: usize) -> Vec<String> {
        (0..count).map(|_| random_string(length)).collect()
    }

    /// Generate a vector of random integers
    pub fn integers(count: usize, min: i32, max: i32) -> Vec<i32> {
        use fake::faker::number::en::*;
        (0..count)
            .map(|_| NumberWithFormat(&format!("#{}-{}", min, max)).fake::<i32>())
            .collect()
    }

    /// Generate random bytes
    pub fn bytes(size: usize) -> Vec<u8> {
        (0..size).map(|_| rand::random::<u8>()).collect()
    }
}

/// Property testing strategies for common types
pub mod strategies {
    use super::*;

    /// Strategy for generating valid identifiers
    pub fn identifier() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]*"
            .prop_map(|s| s)
    }

    /// Strategy for generating file paths
    pub fn file_path() -> impl Strategy<Value = String> {
        prop::collection::vec("[a-zA-Z0-9_.-]+", 1..5)
            .prop_map(|parts| parts.join("/"))
    }

    /// Strategy for generating valid port numbers
    pub fn port() -> impl Strategy<Value = u16> {
        1024u16..65536
    }

    /// Strategy for generating email addresses
    pub fn email() -> impl Strategy<Value = String> {
        ("[a-zA-Z0-9]+", "@", "[a-zA-Z0-9]+", r"\.", "[a-zA-Z]{2,4}")
            .prop_map(|(user, at, domain, dot, tld)| format!("{}{}{}{}{}", user, at, domain, dot, tld))
    }

    /// Strategy for generating JSON-safe strings
    pub fn json_string() -> impl Strategy<Value = String> {
        r#"[a-zA-Z0-9 !@#$%^&*()_+\-=\[\]{}|;':",./<>?`~]*"#
            .prop_map(|s| s)
    }

    /// Strategy for generating non-empty vectors
    pub fn non_empty_vec<T: std::fmt::Debug>(
        element_strategy: impl Strategy<Value = T>,
    ) -> impl Strategy<Value = Vec<T>> {
        prop::collection::vec(element_strategy, 1..10)
    }
}

/// Quick test data generation macros
#[macro_export]
macro_rules! test_data {
    (string) => {
        $crate::generators::random_string(10)
    };
    (string, $len:expr) => {
        $crate::generators::random_string($len)
    };
    (email) => {
        $crate::generators::random_email()
    };
    (uuid) => {
        $crate::generators::random_uuid()
    };
    (port) => {
        $crate::generators::random_port()
    };
    (bytes, $size:expr) => {
        $crate::generators::TestDataGenerator::bytes($size)
    };
}