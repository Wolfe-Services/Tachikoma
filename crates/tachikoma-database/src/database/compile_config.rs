/// Build-time SQLite configuration options
/// These are set when compiling SQLite (if using bundled)

pub struct SqliteCompileOptions {
    /// Maximum number of attached databases
    pub max_attached: u32,
    /// Default page size
    pub default_page_size: u32,
    /// Maximum page size
    pub max_page_size: u32,
    /// Enable FTS5 full-text search
    pub enable_fts5: bool,
    /// Enable JSON1 extension
    pub enable_json1: bool,
    /// Enable R-Tree extension
    pub enable_rtree: bool,
}

impl Default for SqliteCompileOptions {
    fn default() -> Self {
        Self {
            max_attached: 10,
            default_page_size: 4096,
            max_page_size: 65536,
            enable_fts5: true,
            enable_json1: true,
            enable_rtree: false,
        }
    }
}