//! Domain-specific generators for Tachikoma types.

use super::*;

/// Generate a mock API key
pub fn api_key(prefix: &str) -> String {
    format!("{}-{}", prefix, alphanumeric(48))
}

/// Generate a Claude API key
pub fn claude_api_key() -> String {
    api_key("sk-ant")
}

/// Generate an OpenAI API key
pub fn openai_api_key() -> String {
    api_key("sk")
}

/// Generate a session ID
pub fn session_id() -> String {
    format!("session_{}", uuid())
}

/// Generate a mission ID
pub fn mission_id() -> String {
    format!("mission_{}", hex_string(16))
}

/// Generate a spec ID
pub fn spec_id() -> String {
    use rand::Rng;
    let num: u32 = rand::thread_rng().gen_range(1..999);
    format!("{:03}", num)
}

/// User generator
#[derive(Debug, Clone)]
pub struct UserGenerator {
    role: Option<String>,
}

impl UserGenerator {
    pub fn new() -> Self {
        Self { role: None }
    }

    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    pub fn admin(self) -> Self {
        self.with_role("admin")
    }

    pub fn guest(self) -> Self {
        self.with_role("guest")
    }

    pub fn generate(&self) -> GeneratedUser {
        GeneratedUser {
            id: uuid(),
            name: full_name(),
            email: email(),
            username: username(),
            role: self.role.clone().unwrap_or_else(|| "user".into()),
        }
    }

    pub fn generate_many(&self, count: usize) -> Vec<GeneratedUser> {
        (0..count).map(|_| self.generate()).collect()
    }
}

impl Default for UserGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub username: String,
    pub role: String,
}

/// Message generator for chat messages
pub struct MessageGenerator {
    role: String,
}

impl MessageGenerator {
    pub fn user() -> Self {
        Self { role: "user".into() }
    }

    pub fn assistant() -> Self {
        Self { role: "assistant".into() }
    }

    pub fn system() -> Self {
        Self { role: "system".into() }
    }

    pub fn generate(&self) -> GeneratedMessage {
        GeneratedMessage {
            role: self.role.clone(),
            content: paragraph(),
        }
    }

    pub fn generate_conversation(turns: usize) -> Vec<GeneratedMessage> {
        let mut messages = Vec::new();

        // Start with system message
        messages.push(Self::system().generate());

        for i in 0..turns {
            if i % 2 == 0 {
                messages.push(Self::user().generate());
            } else {
                messages.push(Self::assistant().generate());
            }
        }

        messages
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedMessage {
    pub role: String,
    pub content: String,
}

/// Tool call generator
pub struct ToolCallGenerator;

impl ToolCallGenerator {
    pub fn read_file() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "read_file".into(),
            arguments: serde_json::json!({
                "path": file_path()
            }),
        }
    }

    pub fn write_file() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "edit_file".into(),
            arguments: serde_json::json!({
                "path": file_path(),
                "old_string": sentence(),
                "new_string": sentence()
            }),
        }
    }

    pub fn bash() -> GeneratedToolCall {
        let commands = ["ls -la", "pwd", "cat file.txt", "git status", "npm test"];
        use rand::Rng;
        let cmd = commands[rand::thread_rng().gen_range(0..commands.len())];

        GeneratedToolCall {
            name: "bash".into(),
            arguments: serde_json::json!({
                "command": cmd
            }),
        }
    }

    pub fn code_search() -> GeneratedToolCall {
        GeneratedToolCall {
            name: "code_search".into(),
            arguments: serde_json::json!({
                "pattern": words(1)[0],
                "path": dir_path()
            }),
        }
    }

    pub fn random() -> GeneratedToolCall {
        use rand::Rng;
        match rand::thread_rng().gen_range(0..4) {
            0 => Self::read_file(),
            1 => Self::write_file(),
            2 => Self::bash(),
            _ => Self::code_search(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Spec generator
pub struct SpecGenerator;

impl SpecGenerator {
    pub fn generate() -> GeneratedSpec {
        let id = spec_id();
        let title = words(3).join(" ");

        GeneratedSpec {
            id: id.clone(),
            title: title.clone(),
            content: format!(
                "# {} - {}\n\n## Objective\n\n{}\n\n## Acceptance Criteria\n\n- [ ] {}\n- [ ] {}\n- [ ] {}\n",
                id,
                title,
                paragraph(),
                sentence(),
                sentence(),
                sentence()
            ),
            phase: format!("{:02}", rand::thread_rng().gen_range(0..25)),
            status: "Planned".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedSpec {
    pub id: String,
    pub title: String,
    pub content: String,
    pub phase: String,
    pub status: String,
}

/// Batch generator for performance testing
pub fn batch<T, F>(generator: F, count: usize) -> Vec<T>
where
    F: Fn() -> T,
{
    (0..count).map(|_| generator()).collect()
}

/// Generate with seeded context for reproducibility
pub fn with_seed<T, F>(seed: u64, generator: F) -> T
where
    F: FnOnce(&mut GeneratorContext) -> T,
{
    let mut context = GeneratorContext::with_seed(seed);
    generator(&mut context)
}

/// Generate many items with seeded context
pub fn batch_with_seed<T, F>(seed: u64, generator: F, count: usize) -> Vec<T>
where
    F: Fn(&mut GeneratorContext) -> T,
{
    let mut context = GeneratorContext::with_seed(seed);
    (0..count).map(|_| generator(&mut context)).collect()
}