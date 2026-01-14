# Spec 548: Experiment Framework

## Overview
Framework for running controlled experiments on Tachikoma behavior, enabling safe testing of improvements, A/B testing of strategies, and data-driven decision making.


## Acceptance Criteria
- [x] Implementation complete per spec

## Requirements

### Experiment Definition
```go
// Experiment defines a controlled experiment
type Experiment struct {
    ID           string            `json:"id"`
    Name         string            `json:"name"`
    Description  string            `json:"description"`
    Hypothesis   string            `json:"hypothesis"`
    Type         ExperimentType    `json:"type"`
    Control      ExperimentVariant `json:"control"`
    Variants     []ExperimentVariant `json:"variants"`
    Metrics      []ExperimentMetric `json:"metrics"`
    Config       ExperimentConfig  `json:"config"`
    Status       ExperimentStatus  `json:"status"`
    CreatedAt    time.Time         `json:"createdAt"`
    StartedAt    *time.Time        `json:"startedAt,omitempty"`
    EndedAt      *time.Time        `json:"endedAt,omitempty"`
}

type ExperimentType string

const (
    ExperimentTypeAB         ExperimentType = "ab_test"
    ExperimentTypeMultiarm   ExperimentType = "multiarm_bandit"
    ExperimentTypeCanary     ExperimentType = "canary"
    ExperimentTypeShadow     ExperimentType = "shadow"
    ExperimentTypeFeatureFlag ExperimentType = "feature_flag"
)
```

### Experiment Variant
```go
type ExperimentVariant struct {
    ID          string                 `json:"id"`
    Name        string                 `json:"name"`
    Description string                 `json:"description"`
    Config      map[string]interface{} `json:"config"`
    Weight      float64                `json:"weight"` // traffic percentage
    IsControl   bool                   `json:"isControl"`
}
```

### Experiment Manager Interface
```go
type ExperimentManager interface {
    // Create experiment
    Create(ctx context.Context, experiment *Experiment) error

    // Start experiment
    Start(ctx context.Context, experimentID string) error

    // Stop experiment
    Stop(ctx context.Context, experimentID string) error

    // Get experiment status
    Status(ctx context.Context, experimentID string) (*ExperimentStatus, error)

    // Get experiment results
    Results(ctx context.Context, experimentID string) (*ExperimentResults, error)

    // Assign to variant
    Assign(ctx context.Context, experimentID, entityID string) (*ExperimentVariant, error)
}
```

### Experiment Configuration
```go
type ExperimentConfig struct {
    Duration           time.Duration `json:"duration"`
    MinSampleSize      int           `json:"minSampleSize"`
    MaxSampleSize      int           `json:"maxSampleSize"`
    ConfidenceLevel    float64       `json:"confidenceLevel"` // e.g., 0.95
    TrafficPercentage  float64       `json:"trafficPercentage"`
    RampUpPeriod       time.Duration `json:"rampUpPeriod"`
    AutoStop           bool          `json:"autoStop"`
    StopOnSignificance bool          `json:"stopOnSignificance"`
    Guardrails         []Guardrail   `json:"guardrails"`
}
```

### Guardrails
```go
type Guardrail struct {
    Metric    string  `json:"metric"`
    Operator  string  `json:"operator"`
    Threshold float64 `json:"threshold"`
    Action    string  `json:"action"` // stop, alert, rollback
}
```

### Experiment Metrics
```go
type ExperimentMetric struct {
    Name        string     `json:"name"`
    Type        MetricType `json:"type"`
    Goal        string     `json:"goal"` // increase, decrease
    Primary     bool       `json:"primary"`
    MinEffect   float64    `json:"minEffect"` // minimum detectable effect
}

type MetricType string

const (
    MetricTypeConversion MetricType = "conversion" // binary
    MetricTypeContinuous MetricType = "continuous" // numeric
    MetricTypeRatio      MetricType = "ratio"      // rate
)
```

### Statistical Analysis
```go
type StatisticalAnalyzer interface {
    // Calculate statistical significance
    CalculateSignificance(ctx context.Context, results *ExperimentResults) (*SignificanceResult, error)

    // Calculate confidence intervals
    CalculateConfidenceInterval(ctx context.Context, data []float64, level float64) (*ConfidenceInterval, error)

    // Perform power analysis
    PowerAnalysis(ctx context.Context, params PowerParams) (*PowerResult, error)
}

type SignificanceResult struct {
    PValue          float64 `json:"pValue"`
    Significant     bool    `json:"significant"`
    ConfidenceLevel float64 `json:"confidenceLevel"`
    Effect          float64 `json:"effect"`
    EffectSize      string  `json:"effectSize"` // small, medium, large
}
```

### Experiment Results
```go
type ExperimentResults struct {
    ExperimentID    string                     `json:"experimentId"`
    VariantResults  map[string]*VariantResult  `json:"variantResults"`
    Analysis        *SignificanceResult        `json:"analysis"`
    Winner          string                     `json:"winner,omitempty"`
    Recommendation  string                     `json:"recommendation"`
    CollectedAt     time.Time                  `json:"collectedAt"`
}

type VariantResult struct {
    VariantID    string             `json:"variantId"`
    SampleSize   int                `json:"sampleSize"`
    Metrics      map[string]float64 `json:"metrics"`
    Conversions  int                `json:"conversions,omitempty"`
}
```

### Feature Flags
```go
type FeatureFlag interface {
    // Check if feature enabled
    IsEnabled(ctx context.Context, flagName, entityID string) (bool, error)

    // Get flag variant
    GetVariant(ctx context.Context, flagName, entityID string) (string, error)

    // Override for testing
    Override(ctx context.Context, flagName string, enabled bool) error
}
```

## Dependencies
- Spec 545: Self-Improvement Engine
- Spec 549: Advanced Monitoring

## Verification
- [ ] Experiments create correctly
- [ ] Traffic routing works
- [ ] Statistical analysis accurate
- [ ] Guardrails trigger
- [ ] Results collected properly
