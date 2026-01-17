<script lang="ts">
  // Simple mission creation page - can be enhanced later with full dialog
</script>

<div class="mission-creation-page">
  <header class="page-header">
    <h1>Create New Mission</h1>
    <a href="/missions" class="btn-secondary">← Back to Missions</a>
  </header>
  
  <div class="creation-form">
    <div class="form-group">
      <label for="mission-title">Mission Title</label>
      <input 
        id="mission-title" 
        type="text" 
        placeholder="Enter mission title"
        class="form-input"
      />
    </div>
    
    <div class="form-group">
      <label for="mission-prompt">Mission Prompt</label>
      <textarea 
        id="mission-prompt" 
        placeholder="Describe what you want Tachikoma to implement..."
        class="form-textarea"
        rows="8"
      ></textarea>
    </div>
    
    <div class="form-actions">
      <button class="btn-primary">Create Mission</button>
      <button class="btn-secondary">Save as Draft</button>
    </div>
  </div>
</div>

<style>
  .mission-creation-page {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  .page-header h1 {
    font-size: 1.75rem;
    margin: 0;
    color: var(--text-primary);
  }
  
  .creation-form {
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 12px;
    padding: 2rem;
  }
  
  .form-group {
    margin-bottom: 1.5rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
    color: var(--text-primary);
  }
  
  .form-input,
  .form-textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background: var(--bg-primary);
    color: var(--text-primary);
    font-family: inherit;
  }
  
  .form-textarea {
    resize: vertical;
    min-height: 120px;
  }
  
  .form-input:focus,
  .form-textarea:focus {
    outline: none;
    border-color: var(--accent-primary);
  }
  
  .form-actions {
    display: flex;
    gap: 1rem;
    padding-top: 1rem;
  }
  
  .btn-primary {
    background: var(--accent-primary);
    color: white;
    padding: 0.75rem 1.25rem;
    border: none;
    border-radius: 8px;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.2s;
  }
  
  .btn-primary:hover {
    opacity: 0.9;
  }
  
  .btn-secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    padding: 0.75rem 1.25rem;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    text-decoration: none;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }
  
  .btn-secondary:hover {
    background: var(--bg-secondary);
    border-color: var(--accent-primary);
  }
</style>