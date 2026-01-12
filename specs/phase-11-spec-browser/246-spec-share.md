# Spec 246: Spec Sharing

## Phase
11 - Spec Browser UI

## Spec ID
246

## Status
Planned

## Dependencies
- Spec 236 (Spec Detail View)
- Spec 247 (Spec Export)

## Estimated Context
~8%

---

## Objective

Implement sharing functionality for specs including shareable links, access control, embedded views, and collaboration features. Support various sharing formats and destinations.

---

## Acceptance Criteria

- [ ] Generate shareable links
- [ ] Access control (public, private, team)
- [ ] Copy link to clipboard
- [ ] Share via email
- [ ] Embed code generation
- [ ] QR code for mobile sharing
- [ ] Share multiple specs at once
- [ ] Track share analytics
- [ ] Revoke shared access

---

## Implementation Details

### ShareDialog.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { writable } from 'svelte/store';
  import QRCode from 'qrcode';
  import type { Spec, ShareSettings, ShareLink } from '$lib/types/spec';
  import Modal from '$lib/components/Modal.svelte';
  import Button from '$lib/components/Button.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import { generateShareLink, revokeShareLink } from '$lib/api/share';

  export let open = false;
  export let specs: Spec[] = [];

  const dispatch = createEventDispatcher<{
    close: void;
    share: ShareLink;
  }>();

  let activeTab: 'link' | 'embed' | 'email' = 'link';
  let settings = writable<ShareSettings>({
    access: 'anyone',
    expiresIn: null,
    allowComments: true,
    allowDownload: true,
    password: null
  });

  let shareLink: string | null = null;
  let qrCodeDataUrl: string | null = null;
  let copied = false;
  let loading = false;
  let emailAddresses = '';
  let emailMessage = '';

  $: specIds = specs.map(s => s.id);
  $: specTitles = specs.map(s => s.title).join(', ');

  async function generateLink() {
    loading = true;
    try {
      const link = await generateShareLink(specIds, $settings);
      shareLink = link.url;
      dispatch('share', link);

      // Generate QR code
      qrCodeDataUrl = await QRCode.toDataURL(shareLink, {
        width: 200,
        margin: 2,
        color: {
          dark: '#000000',
          light: '#ffffff'
        }
      });
    } catch (e) {
      console.error('Failed to generate share link:', e);
    } finally {
      loading = false;
    }
  }

  async function copyToClipboard() {
    if (shareLink) {
      await navigator.clipboard.writeText(shareLink);
      copied = true;
      setTimeout(() => copied = false, 2000);
    }
  }

  function generateEmbedCode(): string {
    if (!shareLink) return '';

    const width = 600;
    const height = 400;

    return `<iframe
  src="${shareLink}?embed=true"
  width="${width}"
  height="${height}"
  frameborder="0"
  allow="clipboard-write"
  title="Spec ${specIds.join(', ')}"
></iframe>`;
  }

  async function sendEmail() {
    if (!emailAddresses.trim()) return;

    const addresses = emailAddresses.split(',').map(e => e.trim());
    const subject = `Shared Specs: ${specTitles}`;
    const body = `${emailMessage}\n\nView specs: ${shareLink}`;

    // Use mailto link for simplicity
    const mailtoUrl = `mailto:${addresses.join(',')}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
    window.open(mailtoUrl, '_blank');
  }

  function handleClose() {
    shareLink = null;
    qrCodeDataUrl = null;
    dispatch('close');
  }

  // Auto-generate link when dialog opens
  $: if (open && specs.length > 0 && !shareLink) {
    generateLink();
  }
</script>

<Modal {open} on:close={handleClose} size="md" title="Share Specs">
  <div class="share-dialog">
    {#if specs.length > 1}
      <div class="share-dialog__specs">
        <span class="share-dialog__specs-label">Sharing {specs.length} specs:</span>
        <div class="share-dialog__specs-list">
          {#each specs as spec}
            <span class="share-dialog__spec-badge">{spec.id}</span>
          {/each}
        </div>
      </div>
    {:else if specs.length === 1}
      <div class="share-dialog__spec-info">
        <span class="share-dialog__spec-id">{specs[0].id}</span>
        <span class="share-dialog__spec-title">{specs[0].title}</span>
      </div>
    {/if}

    <nav class="share-dialog__tabs">
      <button
        class="share-dialog__tab"
        class:share-dialog__tab--active={activeTab === 'link'}
        on:click={() => activeTab = 'link'}
      >
        <Icon name="link" size={14} />
        Link
      </button>
      <button
        class="share-dialog__tab"
        class:share-dialog__tab--active={activeTab === 'embed'}
        on:click={() => activeTab = 'embed'}
      >
        <Icon name="code" size={14} />
        Embed
      </button>
      <button
        class="share-dialog__tab"
        class:share-dialog__tab--active={activeTab === 'email'}
        on:click={() => activeTab = 'email'}
      >
        <Icon name="mail" size={14} />
        Email
      </button>
    </nav>

    {#if activeTab === 'link'}
      <div class="share-dialog__content">
        <div class="share-dialog__link-section">
          {#if loading}
            <div class="share-dialog__loading">
              <Icon name="loader" size={20} class="spinning" />
              Generating link...
            </div>
          {:else if shareLink}
            <div class="share-dialog__link-input">
              <input type="text" value={shareLink} readonly />
              <Button variant="primary" on:click={copyToClipboard}>
                <Icon name={copied ? 'check' : 'copy'} size={14} />
                {copied ? 'Copied!' : 'Copy'}
              </Button>
            </div>
          {/if}
        </div>

        <div class="share-dialog__settings">
          <h4>Access Settings</h4>

          <div class="share-dialog__setting">
            <label>Who can access</label>
            <select bind:value={$settings.access}>
              <option value="anyone">Anyone with the link</option>
              <option value="team">Team members only</option>
              <option value="specific">Specific people</option>
            </select>
          </div>

          <div class="share-dialog__setting">
            <label>Link expiration</label>
            <select bind:value={$settings.expiresIn}>
              <option value={null}>Never expires</option>
              <option value="1h">1 hour</option>
              <option value="24h">24 hours</option>
              <option value="7d">7 days</option>
              <option value="30d">30 days</option>
            </select>
          </div>

          <div class="share-dialog__setting share-dialog__setting--inline">
            <label>Allow comments</label>
            <Toggle bind:checked={$settings.allowComments} />
          </div>

          <div class="share-dialog__setting share-dialog__setting--inline">
            <label>Allow download</label>
            <Toggle bind:checked={$settings.allowDownload} />
          </div>

          <div class="share-dialog__setting">
            <label>Password protection (optional)</label>
            <input
              type="password"
              bind:value={$settings.password}
              placeholder="Set a password..."
            />
          </div>
        </div>

        {#if qrCodeDataUrl}
          <div class="share-dialog__qr">
            <h4>QR Code</h4>
            <img src={qrCodeDataUrl} alt="QR Code for share link" />
            <Button variant="ghost" size="sm">
              <Icon name="download" size={14} />
              Download QR
            </Button>
          </div>
        {/if}
      </div>
    {:else if activeTab === 'embed'}
      <div class="share-dialog__content">
        <div class="share-dialog__embed">
          <h4>Embed Code</h4>
          <p>Copy this code to embed the spec in your website or documentation.</p>

          <div class="share-dialog__embed-preview">
            <div class="share-dialog__embed-frame">
              <Icon name="layout" size={24} />
              <span>Spec Preview</span>
            </div>
          </div>

          <div class="share-dialog__embed-code">
            <pre><code>{generateEmbedCode()}</code></pre>
            <Button
              variant="ghost"
              size="sm"
              on:click={() => navigator.clipboard.writeText(generateEmbedCode())}
            >
              <Icon name="copy" size={14} />
              Copy code
            </Button>
          </div>
        </div>
      </div>
    {:else if activeTab === 'email'}
      <div class="share-dialog__content">
        <div class="share-dialog__email">
          <div class="share-dialog__field">
            <label for="email-addresses">Email addresses</label>
            <input
              id="email-addresses"
              type="text"
              bind:value={emailAddresses}
              placeholder="email@example.com, another@example.com"
            />
            <span class="share-dialog__field-hint">
              Separate multiple addresses with commas
            </span>
          </div>

          <div class="share-dialog__field">
            <label for="email-message">Message (optional)</label>
            <textarea
              id="email-message"
              bind:value={emailMessage}
              placeholder="Add a personal message..."
              rows="4"
            />
          </div>

          <Button
            variant="primary"
            disabled={!emailAddresses.trim() || !shareLink}
            on:click={sendEmail}
          >
            <Icon name="send" size={14} />
            Send Email
          </Button>
        </div>
      </div>
    {/if}
  </div>

  <svelte:fragment slot="footer">
    <Button variant="outline" on:click={handleClose}>
      Done
    </Button>
  </svelte:fragment>
</Modal>

<style>
  .share-dialog {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .share-dialog__specs {
    padding: 12px 16px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .share-dialog__specs-label {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    display: block;
    margin-bottom: 8px;
  }

  .share-dialog__specs-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .share-dialog__spec-badge {
    padding: 4px 8px;
    font-size: 0.75rem;
    font-weight: 600;
    font-family: var(--font-mono);
    background: var(--color-surface);
    border-radius: 4px;
  }

  .share-dialog__spec-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .share-dialog__spec-id {
    padding: 4px 10px;
    font-family: var(--font-mono);
    font-weight: 600;
    background: var(--color-primary-subtle);
    color: var(--color-primary);
    border-radius: 4px;
  }

  .share-dialog__spec-title {
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .share-dialog__tabs {
    display: flex;
    gap: 4px;
    padding: 4px;
    background: var(--color-surface-subtle);
    border-radius: 8px;
  }

  .share-dialog__tab {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 16px;
    font-size: 0.875rem;
    font-weight: 500;
    background: none;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    color: var(--color-text-secondary);
    transition: all 0.15s;
  }

  .share-dialog__tab:hover {
    background: var(--color-hover);
    color: var(--color-text-primary);
  }

  .share-dialog__tab--active {
    background: var(--color-surface);
    color: var(--color-text-primary);
    box-shadow: var(--shadow-sm);
  }

  .share-dialog__content {
    min-height: 300px;
  }

  .share-dialog__loading {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 16px;
    color: var(--color-text-secondary);
  }

  .share-dialog__link-input {
    display: flex;
    gap: 8px;
    margin-bottom: 20px;
  }

  .share-dialog__link-input input {
    flex: 1;
    padding: 10px 14px;
    font-size: 0.875rem;
    font-family: var(--font-mono);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-surface-subtle);
  }

  .share-dialog__settings h4 {
    font-size: 0.875rem;
    font-weight: 600;
    color: var(--color-text-secondary);
    margin: 0 0 16px;
  }

  .share-dialog__setting {
    margin-bottom: 16px;
  }

  .share-dialog__setting label {
    display: block;
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin-bottom: 6px;
  }

  .share-dialog__setting select,
  .share-dialog__setting input {
    width: 100%;
    padding: 8px 12px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
  }

  .share-dialog__setting--inline {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .share-dialog__setting--inline label {
    margin-bottom: 0;
  }

  .share-dialog__qr {
    margin-top: 20px;
    padding-top: 20px;
    border-top: 1px solid var(--color-border);
    text-align: center;
  }

  .share-dialog__qr h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 12px;
  }

  .share-dialog__qr img {
    border-radius: 8px;
    margin-bottom: 12px;
  }

  .share-dialog__embed h4 {
    font-size: 0.875rem;
    font-weight: 600;
    margin: 0 0 8px;
  }

  .share-dialog__embed > p {
    font-size: 0.875rem;
    color: var(--color-text-secondary);
    margin: 0 0 16px;
  }

  .share-dialog__embed-preview {
    padding: 40px;
    background: var(--color-surface-subtle);
    border: 1px dashed var(--color-border);
    border-radius: 8px;
    margin-bottom: 16px;
  }

  .share-dialog__embed-frame {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--color-text-tertiary);
    font-size: 0.875rem;
  }

  .share-dialog__embed-code {
    position: relative;
  }

  .share-dialog__embed-code pre {
    padding: 16px;
    background: var(--color-code-bg);
    border-radius: 6px;
    overflow-x: auto;
  }

  .share-dialog__embed-code code {
    font-family: var(--font-mono);
    font-size: 0.75rem;
    color: var(--color-text-primary);
  }

  .share-dialog__embed-code :global(button) {
    position: absolute;
    top: 8px;
    right: 8px;
  }

  .share-dialog__email {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .share-dialog__field label {
    display: block;
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 6px;
  }

  .share-dialog__field input,
  .share-dialog__field textarea {
    width: 100%;
    padding: 10px 14px;
    font-size: 0.875rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-family: inherit;
  }

  .share-dialog__field-hint {
    font-size: 0.75rem;
    color: var(--color-text-tertiary);
    margin-top: 4px;
  }
</style>
```

### Share Types

```typescript
// types/spec.ts additions
export type ShareAccess = 'anyone' | 'team' | 'specific';

export interface ShareSettings {
  access: ShareAccess;
  expiresIn: string | null;
  allowComments: boolean;
  allowDownload: boolean;
  password: string | null;
  specificUsers?: string[];
}

export interface ShareLink {
  id: string;
  url: string;
  specIds: string[];
  settings: ShareSettings;
  createdAt: Date;
  createdBy: string;
  accessCount: number;
  lastAccessedAt?: Date;
}
```

---

## Testing Requirements

### Unit Tests

```typescript
import { render, fireEvent, screen, waitFor } from '@testing-library/svelte';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import ShareDialog from './ShareDialog.svelte';
import * as shareApi from '$lib/api/share';

describe('ShareDialog', () => {
  const mockSpec = {
    id: '246',
    title: 'Spec Sharing',
    status: 'planned'
  };

  beforeEach(() => {
    vi.spyOn(shareApi, 'generateShareLink').mockResolvedValue({
      id: 'share-1',
      url: 'https://example.com/share/abc123',
      specIds: ['246'],
      settings: {},
      createdAt: new Date(),
      createdBy: 'Test',
      accessCount: 0
    });
  });

  it('generates share link on open', async () => {
    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    await waitFor(() => {
      expect(screen.getByDisplayValue(/example\.com/)).toBeInTheDocument();
    });
  });

  it('copies link to clipboard', async () => {
    const mockClipboard = { writeText: vi.fn() };
    Object.assign(navigator, { clipboard: mockClipboard });

    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    await waitFor(() => {
      expect(screen.getByText('Copy')).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Copy'));

    expect(mockClipboard.writeText).toHaveBeenCalled();
    expect(screen.getByText('Copied!')).toBeInTheDocument();
  });

  it('switches between tabs', async () => {
    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    await fireEvent.click(screen.getByText('Embed'));
    expect(screen.getByText('Embed Code')).toBeInTheDocument();

    await fireEvent.click(screen.getByText('Email'));
    expect(screen.getByText('Email addresses')).toBeInTheDocument();
  });

  it('shows multiple specs when sharing many', () => {
    const specs = [
      { id: '246', title: 'Spec 1' },
      { id: '247', title: 'Spec 2' }
    ];

    render(ShareDialog, { props: { open: true, specs } });

    expect(screen.getByText('Sharing 2 specs:')).toBeInTheDocument();
  });

  it('updates access settings', async () => {
    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    const accessSelect = screen.getByLabelText(/Who can access/);
    await fireEvent.change(accessSelect, { target: { value: 'team' } });

    // Settings should be updated
  });

  it('generates embed code', async () => {
    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    await waitFor(() => {
      expect(screen.getByDisplayValue(/example\.com/)).toBeInTheDocument();
    });

    await fireEvent.click(screen.getByText('Embed'));

    expect(screen.getByText(/iframe/)).toBeInTheDocument();
  });

  it('handles email form', async () => {
    render(ShareDialog, { props: { open: true, specs: [mockSpec] } });

    await fireEvent.click(screen.getByText('Email'));

    const emailInput = screen.getByLabelText('Email addresses');
    await fireEvent.input(emailInput, { target: { value: 'test@example.com' } });

    expect(screen.getByText('Send Email')).not.toBeDisabled();
  });

  it('dispatches share event', async () => {
    const { component } = render(ShareDialog, {
      props: { open: true, specs: [mockSpec] }
    });

    const shareHandler = vi.fn();
    component.$on('share', shareHandler);

    await waitFor(() => {
      expect(shareHandler).toHaveBeenCalled();
    });
  });
});
```

---

## Related Specs

- Spec 236: Spec Detail View
- Spec 247: Spec Export
- Spec 249: Batch Operations
