# Spec 544: Knowledge Base System

## Overview
Persistent knowledge base for Tachikoma agents to store, retrieve, and share learned patterns, solutions, and contextual information for self-improvement.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Knowledge Entry Types
```go
// KnowledgeEntry represents a piece of stored knowledge
type KnowledgeEntry struct {
    ID          string            `json:"id"`
    Type        KnowledgeType     `json:"type"`
    Title       string            `json:"title"`
    Content     string            `json:"content"`
    Embedding   []float32         `json:"embedding,omitempty"`
    Metadata    map[string]string `json:"metadata"`
    Tags        []string          `json:"tags"`
    Source      string            `json:"source"`
    Confidence  float64           `json:"confidence"`
    UsageCount  int64             `json:"usageCount"`
    CreatedAt   time.Time         `json:"createdAt"`
    UpdatedAt   time.Time         `json:"updatedAt"`
    ExpiresAt   *time.Time        `json:"expiresAt,omitempty"`
}

type KnowledgeType string

const (
    KnowledgeTypePattern   KnowledgeType = "pattern"
    KnowledgeTypeSolution  KnowledgeType = "solution"
    KnowledgeTypeError     KnowledgeType = "error"
    KnowledgeTypeProcedure KnowledgeType = "procedure"
    KnowledgeTypeFact      KnowledgeType = "fact"
    KnowledgeTypeContext   KnowledgeType = "context"
)
```

### Knowledge Base Interface
```go
type KnowledgeBase interface {
    // Store knowledge
    Store(ctx context.Context, entry *KnowledgeEntry) error

    // Retrieve by ID
    Get(ctx context.Context, id string) (*KnowledgeEntry, error)

    // Search by text
    Search(ctx context.Context, query string, opts SearchOptions) ([]*KnowledgeEntry, error)

    // Semantic search
    SemanticSearch(ctx context.Context, embedding []float32, opts SearchOptions) ([]*KnowledgeEntry, error)

    // Update entry
    Update(ctx context.Context, entry *KnowledgeEntry) error

    // Delete entry
    Delete(ctx context.Context, id string) error

    // Record usage
    RecordUsage(ctx context.Context, id string, outcome UsageOutcome) error
}
```

### Search Options
```go
type SearchOptions struct {
    Types       []KnowledgeType   `json:"types,omitempty"`
    Tags        []string          `json:"tags,omitempty"`
    MinConfidence float64         `json:"minConfidence"`
    Limit       int               `json:"limit"`
    Offset      int               `json:"offset"`
    SortBy      string            `json:"sortBy"`
    TimeRange   *TimeRange        `json:"timeRange,omitempty"`
}
```

### Embedding Service
```go
type EmbeddingService interface {
    // Generate embedding for text
    Embed(ctx context.Context, text string) ([]float32, error)

    // Batch embedding
    BatchEmbed(ctx context.Context, texts []string) ([][]float32, error)

    // Compute similarity
    Similarity(a, b []float32) float64
}
```

### Knowledge Extraction
```go
type KnowledgeExtractor interface {
    // Extract knowledge from task execution
    ExtractFromTask(ctx context.Context, task *TaskResult) ([]*KnowledgeEntry, error)

    // Extract from error
    ExtractFromError(ctx context.Context, err error, context map[string]interface{}) (*KnowledgeEntry, error)

    // Extract patterns from history
    ExtractPatterns(ctx context.Context, history []*TaskResult) ([]*KnowledgeEntry, error)
}
```

### Knowledge Sharing
```go
type KnowledgeSharing interface {
    // Share entry with other agents
    Share(ctx context.Context, entryID string, targetAgents []string) error

    // Receive shared knowledge
    Receive(ctx context.Context, entry *KnowledgeEntry, fromAgent string) error

    // Sync with knowledge hub
    Sync(ctx context.Context) error
}
```

### Retention Policies
```go
type RetentionPolicy struct {
    MaxEntries      int           `json:"maxEntries"`
    MaxAge          time.Duration `json:"maxAge"`
    MinConfidence   float64       `json:"minConfidence"`
    MinUsageCount   int64         `json:"minUsageCount"`
    PreserveTypes   []KnowledgeType `json:"preserveTypes"`
}
```

### Knowledge Validation
```go
type KnowledgeValidator interface {
    // Validate entry accuracy
    Validate(ctx context.Context, entry *KnowledgeEntry) (*ValidationResult, error)

    // Cross-reference with other sources
    CrossReference(ctx context.Context, entry *KnowledgeEntry) ([]*Reference, error)
}
```

### Storage Backend
- SQLite with FTS5 for full-text search
- Vector database (pgvector, Milvus) for semantic search
- Redis for caching hot entries
- S3/GCS for large content

## Dependencies
- None (foundational service)

## Verification
- [ ] CRUD operations work
- [ ] Full-text search accurate
- [ ] Semantic search functional
- [ ] Knowledge sharing works
- [ ] Retention policies enforce
