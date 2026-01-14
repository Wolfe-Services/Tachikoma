// src/database/migration/cli.rs

use super::runner::MigrationRunner;
use super::types::*;
use clap::{Parser, Subcommand};
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;
use tokio::fs;
use chrono::Utc;
use tracing::info;

#[derive(Parser)]
#[command(name = "migrate")]
#[command(about = "Database migration management")]
pub struct MigrateCli {
    #[command(subcommand)]
    pub command: MigrateCommand,

    /// Database URL
    #[arg(long, env = "DATABASE_URL", default_value = "sqlite:tachikoma.db")]
    pub database_url: String,

    /// Migrations directory
    #[arg(long, default_value = "migrations")]
    pub migrations_dir: PathBuf,
}

#[derive(Subcommand)]
pub enum MigrateCommand {
    /// Create a new migration
    Create {
        /// Migration name
        name: String,
    },

    /// Run all pending migrations
    Up {
        /// Run only up to this version
        #[arg(long)]
        to: Option<i64>,
    },

    /// Rollback migrations
    Down {
        /// Number of migrations to rollback
        #[arg(default_value = "1")]
        count: u32,

        /// Rollback to specific version
        #[arg(long)]
        to: Option<i64>,
    },

    /// Show migration status
    Status,

    /// Verify migration checksums
    Verify,

    /// Reset database (dangerous!)
    Reset {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Generate migration from schema diff
    Diff {
        /// Target schema file
        schema: PathBuf,
    },
}

impl MigrateCli {
    pub async fn run(&self) -> Result<(), MigrationError> {
        match &self.command {
            MigrateCommand::Create { name } => self.create_migration(name).await,
            MigrateCommand::Up { to } => self.run_up(*to).await,
            MigrateCommand::Down { count, to } => self.run_down(*count, *to).await,
            MigrateCommand::Status => self.show_status().await,
            MigrateCommand::Verify => self.verify_migrations().await,
            MigrateCommand::Reset { force } => self.reset_database(*force).await,
            MigrateCommand::Diff { schema } => self.generate_diff(schema).await,
        }
    }

    async fn create_migration(&self, name: &str) -> Result<(), MigrationError> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let filename = format!("{}_{}.sql", timestamp, sanitize_name(name));
        let path = self.migrations_dir.join(&filename);

        // Ensure migrations directory exists
        fs::create_dir_all(&self.migrations_dir).await?;

        let template = format!(r#"-- Migration: {name}
-- Created: {timestamp}

-- UP
-- Add your migration SQL here


-- DOWN
-- Add rollback SQL here (optional but recommended)

"#);

        fs::write(&path, template).await?;

        println!("Created migration: {}", path.display());
        println!("Edit the file to add your migration SQL.");

        Ok(())
    }

    async fn run_up(&self, to: Option<i64>) -> Result<(), MigrationError> {
        let pool = self.connect().await?;
        let mut runner = MigrationRunner::new(pool);

        // Load migrations from directory
        let migrations = self.load_migrations().await?;
        runner.add_migrations(migrations);

        let plan = runner.plan(to).await?;

        if plan.is_empty() {
            println!("No pending migrations.");
            return Ok(());
        }

        println!("Running {} migration(s)...", plan.len());

        let results = match to {
            Some(version) => runner.run_to(version).await?,
            None => runner.run().await?,
        };

        for result in results {
            println!(
                "  [OK] {} - {} ({}ms)",
                result.version,
                result.name,
                result.execution_time_ms
            );
        }

        println!("Migrations completed successfully.");
        Ok(())
    }

    async fn run_down(&self, count: u32, to: Option<i64>) -> Result<(), MigrationError> {
        let pool = self.connect().await?;
        let mut runner = MigrationRunner::new(pool);

        let migrations = self.load_migrations().await?;
        runner.add_migrations(migrations);

        let current = runner.current_version().await?;

        let target = match (to, current) {
            (Some(t), _) => t,
            (None, Some(cur)) => {
                // Calculate target version based on count
                let applied = runner.get_applied().await?;
                if applied.len() < count as usize {
                    0
                } else {
                    applied[applied.len() - count as usize].version - 1
                }
            }
            (None, None) => {
                println!("No migrations to rollback.");
                return Ok(());
            }
        };

        let plan = runner.plan(Some(target)).await?;

        if plan.is_empty() {
            println!("No migrations to rollback.");
            return Ok(());
        }

        println!("Rolling back {} migration(s)...", plan.len());

        let results = runner.run_to(target).await?;

        for result in results {
            println!(
                "  [ROLLED BACK] {} - {} ({}ms)",
                result.version,
                result.name,
                result.execution_time_ms
            );
        }

        println!("Rollback completed successfully.");
        Ok(())
    }

    async fn show_status(&self) -> Result<(), MigrationError> {
        let pool = self.connect().await?;
        let mut runner = MigrationRunner::new(pool);

        let migrations = self.load_migrations().await?;
        runner.add_migrations(migrations);
        runner.init().await?;

        let applied = runner.get_applied().await?;
        let pending = runner.pending().await?;

        println!("Migration Status");
        println!("================");
        println!();

        if applied.is_empty() && pending.is_empty() {
            println!("No migrations found.");
            return Ok(());
        }

        println!("Applied migrations:");
        if applied.is_empty() {
            println!("  (none)");
        } else {
            for m in &applied {
                println!(
                    "  [x] {} - {} (applied: {})",
                    m.version,
                    m.name,
                    m.applied_at.format("%Y-%m-%d %H:%M:%S")
                );
            }
        }

        println!();
        println!("Pending migrations:");
        if pending.is_empty() {
            println!("  (none)");
        } else {
            for m in &pending {
                println!("  [ ] {} - {}", m.version, m.name);
            }
        }

        println!();
        println!(
            "Summary: {} applied, {} pending",
            applied.len(),
            pending.len()
        );

        Ok(())
    }

    async fn verify_migrations(&self) -> Result<(), MigrationError> {
        let pool = self.connect().await?;
        let mut runner = MigrationRunner::new(pool);

        let migrations = self.load_migrations().await?;
        runner.add_migrations(migrations);
        runner.init().await?;

        let mismatches = runner.verify().await?;

        if mismatches.is_empty() {
            println!("All migration checksums verified successfully.");
        } else {
            println!("Checksum mismatches found:");
            for mismatch in &mismatches {
                println!("  - {}", mismatch);
            }
            return Err(MigrationError::ChecksumMismatch(
                "One or more checksums do not match".to_string()
            ));
        }

        Ok(())
    }

    async fn reset_database(&self, force: bool) -> Result<(), MigrationError> {
        if !force {
            println!("WARNING: This will drop all tables and re-run migrations!");
            println!("Use --force to confirm.");
            return Ok(());
        }

        let pool = self.connect().await?;

        // Get all tables
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'"
        )
        .fetch_all(&pool)
        .await?;

        // Drop all tables
        for (table,) in tables {
            sqlx::query(&format!("DROP TABLE IF EXISTS {}", table))
                .execute(&pool)
                .await?;
        }

        println!("Database reset. Running migrations...");

        // Re-run migrations
        self.run_up(None).await
    }

    async fn generate_diff(&self, _schema: &PathBuf) -> Result<(), MigrationError> {
        println!("Schema diff generation not yet implemented.");
        println!("This feature will compare current database schema with target schema file.");
        Ok(())
    }

    async fn connect(&self) -> Result<SqlitePool, MigrationError> {
        let pool = SqlitePool::connect(&self.database_url).await?;
        Ok(pool)
    }

    async fn load_migrations(&self) -> Result<Vec<Migration>, MigrationError> {
        let mut migrations = Vec::new();

        if !self.migrations_dir.exists() {
            return Ok(migrations);
        }

        let mut entries = fs::read_dir(&self.migrations_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "sql") {
                if let Some(migration) = parse_migration_file(&path).await? {
                    migrations.push(migration);
                }
            }
        }

        migrations.sort_by_key(|m| m.version);
        Ok(migrations)
    }
}

fn sanitize_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

async fn parse_migration_file(path: &PathBuf) -> Result<Option<Migration>, MigrationError> {
    let filename = path.file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| MigrationError::InvalidFormat("Invalid filename".to_string()))?;

    // Parse version from filename (format: YYYYMMDDHHMMSS_name)
    let parts: Vec<&str> = filename.splitn(2, '_').collect();
    if parts.len() != 2 {
        return Err(MigrationError::InvalidFormat(
            format!("Invalid migration filename format: {}", filename)
        ));
    }

    let version: i64 = parts[0].parse()
        .map_err(|_| MigrationError::InvalidFormat(
            format!("Invalid version number: {}", parts[0])
        ))?;

    let name = parts[1].to_string();

    let content = fs::read_to_string(path).await?;

    // Parse UP and DOWN sections
    let (up_sql, down_sql) = parse_migration_content(&content)?;

    let mut migration = Migration::new(version, name, up_sql);
    if let Some(down) = down_sql {
        migration = migration.with_down(down);
    }

    Ok(Some(migration))
}

fn parse_migration_content(content: &str) -> Result<(String, Option<String>), MigrationError> {
    let content = content.trim();

    // Look for -- UP and -- DOWN markers
    let up_marker = content.find("-- UP");
    let down_marker = content.find("-- DOWN");

    match (up_marker, down_marker) {
        (Some(up_start), Some(down_start)) => {
            let up_content_start = content[up_start..].find('\n')
                .map(|i| up_start + i + 1)
                .unwrap_or(up_start + 5);

            let up_sql = content[up_content_start..down_start].trim().to_string();

            let down_content_start = content[down_start..].find('\n')
                .map(|i| down_start + i + 1)
                .unwrap_or(down_start + 7);

            let down_sql = content[down_content_start..].trim().to_string();
            let down_sql = if down_sql.is_empty() { None } else { Some(down_sql) };

            Ok((up_sql, down_sql))
        }
        (Some(up_start), None) => {
            let up_content_start = content[up_start..].find('\n')
                .map(|i| up_start + i + 1)
                .unwrap_or(up_start + 5);

            let up_sql = content[up_content_start..].trim().to_string();
            Ok((up_sql, None))
        }
        (None, _) => {
            // No markers, treat entire content as UP migration
            Ok((content.to_string(), None))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("create_users"), "create_users");
        assert_eq!(sanitize_name("Create Users Table"), "create_users_table");
        assert_eq!(sanitize_name("add-email-column"), "add_email_column");
    }

    #[test]
    fn test_parse_migration_content() {
        let content = r#"
-- UP
CREATE TABLE users (id INTEGER PRIMARY KEY);

-- DOWN
DROP TABLE users;
"#;

        let (up, down) = parse_migration_content(content).unwrap();
        assert!(up.contains("CREATE TABLE"));
        assert!(down.unwrap().contains("DROP TABLE"));
    }

    #[test]
    fn test_parse_migration_content_no_down() {
        let content = r#"
-- UP
CREATE TABLE users (id INTEGER PRIMARY KEY);
"#;

        let (up, down) = parse_migration_content(content).unwrap();
        assert!(up.contains("CREATE TABLE"));
        assert!(down.is_none());
    }
}