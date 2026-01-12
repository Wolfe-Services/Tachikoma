<script lang="ts">
  import type { CoverageReport } from '$lib/types/test-results';

  export let coverage: CoverageReport;

  function formatPercentage(percentage: number): string {
    return percentage.toFixed(1);
  }

  function getCoverageColor(percentage: number): string {
    if (percentage >= 80) return 'var(--color-success)';
    if (percentage >= 60) return 'var(--color-warning)';
    return 'var(--color-error)';
  }
</script>

<div class="coverage-summary">
  <h4 class="coverage-summary__title">Code Coverage</h4>
  
  <div class="coverage-grid">
    <div class="coverage-metric">
      <div class="coverage-metric__header">
        <span class="coverage-metric__label">Lines</span>
        <span class="coverage-metric__percentage" style="color: {getCoverageColor(coverage.lines.percentage)}">
          {formatPercentage(coverage.lines.percentage)}%
        </span>
      </div>
      <div class="coverage-metric__bar">
        <div 
          class="coverage-metric__fill"
          style="width: {coverage.lines.percentage}%; background-color: {getCoverageColor(coverage.lines.percentage)}"
        ></div>
      </div>
      <div class="coverage-metric__stats">
        {coverage.lines.covered} / {coverage.lines.total}
      </div>
    </div>

    <div class="coverage-metric">
      <div class="coverage-metric__header">
        <span class="coverage-metric__label">Branches</span>
        <span class="coverage-metric__percentage" style="color: {getCoverageColor(coverage.branches.percentage)}">
          {formatPercentage(coverage.branches.percentage)}%
        </span>
      </div>
      <div class="coverage-metric__bar">
        <div 
          class="coverage-metric__fill"
          style="width: {coverage.branches.percentage}%; background-color: {getCoverageColor(coverage.branches.percentage)}"
        ></div>
      </div>
      <div class="coverage-metric__stats">
        {coverage.branches.covered} / {coverage.branches.total}
      </div>
    </div>

    <div class="coverage-metric">
      <div class="coverage-metric__header">
        <span class="coverage-metric__label">Functions</span>
        <span class="coverage-metric__percentage" style="color: {getCoverageColor(coverage.functions.percentage)}">
          {formatPercentage(coverage.functions.percentage)}%
        </span>
      </div>
      <div class="coverage-metric__bar">
        <div 
          class="coverage-metric__fill"
          style="width: {coverage.functions.percentage}%; background-color: {getCoverageColor(coverage.functions.percentage)}"
        ></div>
      </div>
      <div class="coverage-metric__stats">
        {coverage.functions.covered} / {coverage.functions.total}
      </div>
    </div>

    <div class="coverage-metric">
      <div class="coverage-metric__header">
        <span class="coverage-metric__label">Statements</span>
        <span class="coverage-metric__percentage" style="color: {getCoverageColor(coverage.statements.percentage)}">
          {formatPercentage(coverage.statements.percentage)}%
        </span>
      </div>
      <div class="coverage-metric__bar">
        <div 
          class="coverage-metric__fill"
          style="width: {coverage.statements.percentage}%; background-color: {getCoverageColor(coverage.statements.percentage)}"
        ></div>
      </div>
      <div class="coverage-metric__stats">
        {coverage.statements.covered} / {coverage.statements.total}
      </div>
    </div>
  </div>
</div>

<style>
  .coverage-summary {
    margin-bottom: 16px;
    padding: 16px;
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .coverage-summary__title {
    margin: 0 0 12px 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .coverage-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
  }

  .coverage-metric {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .coverage-metric__header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .coverage-metric__label {
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .coverage-metric__percentage {
    font-size: 13px;
    font-weight: 600;
  }

  .coverage-metric__bar {
    height: 8px;
    background: var(--color-bg-primary);
    border-radius: 4px;
    overflow: hidden;
    border: 1px solid var(--color-border);
  }

  .coverage-metric__fill {
    height: 100%;
    transition: width 0.3s ease;
  }

  .coverage-metric__stats {
    font-size: 11px;
    color: var(--color-text-muted);
    text-align: right;
  }
</style>