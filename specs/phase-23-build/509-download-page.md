# 509 - Download Page

**Phase:** 23 - Build & Distribution
**Spec ID:** 509
**Status:** Planned
**Dependencies:** 508-distribution-cdn
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Create a user-friendly download page that automatically detects the user's platform and provides appropriate download links with verification instructions.

---

## Acceptance Criteria

- [ ] Platform auto-detection working
- [ ] Download links for all platforms
- [ ] Version information displayed
- [ ] Checksums available
- [ ] Installation instructions provided
- [ ] System requirements listed

---

## Implementation Details

### 1. Download Page Component

Create `web/src/routes/download/+page.svelte`:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { browser } from '$app/environment';

  interface Release {
    version: string;
    date: string;
    downloads: {
      macos_arm64: string;
      macos_x64: string;
      windows: string;
      linux_appimage: string;
      linux_deb: string;
    };
    checksums: Record<string, string>;
  }

  let platform: 'macos' | 'windows' | 'linux' | 'unknown' = 'unknown';
  let arch: 'arm64' | 'x64' = 'x64';
  let release: Release | null = null;
  let loading = true;

  onMount(async () => {
    // Detect platform
    if (browser) {
      const userAgent = navigator.userAgent.toLowerCase();
      const platform_info = navigator.platform?.toLowerCase() || '';

      if (userAgent.includes('mac') || platform_info.includes('mac')) {
        platform = 'macos';
        // Detect Apple Silicon
        if (platform_info.includes('arm') || userAgent.includes('arm64')) {
          arch = 'arm64';
        }
      } else if (userAgent.includes('win') || platform_info.includes('win')) {
        platform = 'windows';
      } else if (userAgent.includes('linux')) {
        platform = 'linux';
      }
    }

    // Fetch latest release
    try {
      const response = await fetch('/api/releases/latest');
      release = await response.json();
    } catch (e) {
      console.error('Failed to fetch release info:', e);
    }

    loading = false;
  });

  function getPrimaryDownload(): { url: string; label: string } | null {
    if (!release) return null;

    switch (platform) {
      case 'macos':
        return {
          url: arch === 'arm64' ? release.downloads.macos_arm64 : release.downloads.macos_x64,
          label: `Download for macOS (${arch === 'arm64' ? 'Apple Silicon' : 'Intel'})`
        };
      case 'windows':
        return {
          url: release.downloads.windows,
          label: 'Download for Windows'
        };
      case 'linux':
        return {
          url: release.downloads.linux_appimage,
          label: 'Download for Linux (AppImage)'
        };
      default:
        return null;
    }
  }

  $: primaryDownload = getPrimaryDownload();
</script>

<svelte:head>
  <title>Download Tachikoma</title>
  <meta name="description" content="Download Tachikoma - AI-powered development assistant for your desktop" />
</svelte:head>

<div class="download-page">
  <header class="hero">
    <h1>Download Tachikoma</h1>
    <p class="tagline">AI-powered autonomous development for your desktop</p>
  </header>

  {#if loading}
    <div class="loading">
      <span class="spinner"></span>
      <p>Loading download options...</p>
    </div>
  {:else if release}
    <section class="primary-download">
      {#if primaryDownload}
        <a href={primaryDownload.url} class="download-button primary">
          <span class="icon">⬇</span>
          <span class="text">
            <strong>{primaryDownload.label}</strong>
            <small>Version {release.version}</small>
          </span>
        </a>
      {:else}
        <p>Select your platform below</p>
      {/if}
    </section>

    <section class="all-downloads">
      <h2>All Downloads</h2>

      <div class="platform-grid">
        <!-- macOS -->
        <div class="platform-card">
          <h3>macOS</h3>
          <div class="downloads">
            <a href={release.downloads.macos_arm64} class="download-link">
              <span class="filename">Tachikoma-{release.version}-arm64.dmg</span>
              <span class="badge">Apple Silicon</span>
            </a>
            <a href={release.downloads.macos_x64} class="download-link">
              <span class="filename">Tachikoma-{release.version}-x64.dmg</span>
              <span class="badge">Intel</span>
            </a>
          </div>
          <details class="requirements">
            <summary>System Requirements</summary>
            <ul>
              <li>macOS 11 Big Sur or later</li>
              <li>4 GB RAM minimum (8 GB recommended)</li>
              <li>500 MB disk space</li>
            </ul>
          </details>
        </div>

        <!-- Windows -->
        <div class="platform-card">
          <h3>Windows</h3>
          <div class="downloads">
            <a href={release.downloads.windows} class="download-link">
              <span class="filename">Tachikoma-Setup-{release.version}.exe</span>
              <span class="badge">64-bit</span>
            </a>
          </div>
          <details class="requirements">
            <summary>System Requirements</summary>
            <ul>
              <li>Windows 10 (version 1903+) or Windows 11</li>
              <li>4 GB RAM minimum (8 GB recommended)</li>
              <li>500 MB disk space</li>
            </ul>
          </details>
        </div>

        <!-- Linux -->
        <div class="platform-card">
          <h3>Linux</h3>
          <div class="downloads">
            <a href={release.downloads.linux_appimage} class="download-link">
              <span class="filename">Tachikoma-{release.version}.AppImage</span>
              <span class="badge">Universal</span>
            </a>
            <a href={release.downloads.linux_deb} class="download-link">
              <span class="filename">tachikoma_{release.version}_amd64.deb</span>
              <span class="badge">Debian/Ubuntu</span>
            </a>
          </div>
          <details class="requirements">
            <summary>System Requirements</summary>
            <ul>
              <li>Ubuntu 20.04+, Debian 11+, or equivalent</li>
              <li>4 GB RAM minimum (8 GB recommended)</li>
              <li>500 MB disk space</li>
              <li>GLIBC 2.31 or later</li>
            </ul>
          </details>
        </div>
      </div>
    </section>

    <section class="verification">
      <h2>Verify Your Download</h2>
      <p>Verify the integrity of your download using SHA-256 checksums:</p>

      <div class="checksums">
        <pre>{#each Object.entries(release.checksums) as [file, hash]}
{hash}  {file}
{/each}</pre>
      </div>

      <details class="verify-instructions">
        <summary>How to verify</summary>
        <div class="instructions">
          <h4>macOS / Linux</h4>
          <pre><code>shasum -a 256 Tachikoma-*.dmg</code></pre>

          <h4>Windows (PowerShell)</h4>
          <pre><code>Get-FileHash Tachikoma-Setup-*.exe -Algorithm SHA256</code></pre>
        </div>
      </details>
    </section>

    <section class="installation">
      <h2>Installation Instructions</h2>

      <div class="install-tabs">
        <details open>
          <summary>macOS</summary>
          <ol>
            <li>Download the DMG file for your Mac</li>
            <li>Open the DMG file</li>
            <li>Drag Tachikoma to the Applications folder</li>
            <li>Open Tachikoma from Applications</li>
            <li>If prompted, click "Open" to allow the app from an identified developer</li>
          </ol>
        </details>

        <details>
          <summary>Windows</summary>
          <ol>
            <li>Download the installer (.exe)</li>
            <li>Run the installer</li>
            <li>Follow the installation wizard</li>
            <li>Launch Tachikoma from the Start Menu or Desktop shortcut</li>
          </ol>
        </details>

        <details>
          <summary>Linux (AppImage)</summary>
          <ol>
            <li>Download the AppImage</li>
            <li>Make it executable: <code>chmod +x Tachikoma-*.AppImage</code></li>
            <li>Run: <code>./Tachikoma-*.AppImage</code></li>
          </ol>
        </details>

        <details>
          <summary>Linux (Debian/Ubuntu)</summary>
          <ol>
            <li>Download the .deb package</li>
            <li>Install: <code>sudo dpkg -i tachikoma_*.deb</code></li>
            <li>If dependencies are missing: <code>sudo apt-get install -f</code></li>
            <li>Launch from applications menu or run: <code>tachikoma</code></li>
          </ol>
        </details>
      </div>
    </section>

    <section class="alternative-methods">
      <h2>Alternative Installation Methods</h2>

      <div class="method">
        <h3>Homebrew (macOS)</h3>
        <pre><code>brew install --cask tachikoma</code></pre>
      </div>

      <div class="method">
        <h3>Snap (Linux)</h3>
        <pre><code>snap install tachikoma</code></pre>
      </div>
    </section>

    <footer class="release-info">
      <p>
        Version {release.version} released on {release.date}
        | <a href="/changelog">Changelog</a>
        | <a href="/release-notes/v{release.version}">Release Notes</a>
        | <a href="https://github.com/tachikoma/tachikoma/releases">All Releases</a>
      </p>
    </footer>
  {:else}
    <div class="error">
      <p>Unable to load release information.</p>
      <p>
        Please visit our
        <a href="https://github.com/tachikoma/tachikoma/releases">GitHub Releases</a>
        page to download directly.
      </p>
    </div>
  {/if}
</div>

<style>
  .download-page {
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem;
  }

  .hero {
    text-align: center;
    margin-bottom: 3rem;
  }

  .hero h1 {
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  .tagline {
    color: var(--color-text-secondary);
    font-size: 1.25rem;
  }

  .primary-download {
    text-align: center;
    margin-bottom: 3rem;
  }

  .download-button {
    display: inline-flex;
    align-items: center;
    gap: 1rem;
    padding: 1rem 2rem;
    background: var(--color-primary);
    color: white;
    border-radius: 8px;
    text-decoration: none;
    font-size: 1.1rem;
    transition: transform 0.2s, box-shadow 0.2s;
  }

  .download-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .download-button .text {
    display: flex;
    flex-direction: column;
    text-align: left;
  }

  .download-button small {
    font-size: 0.85rem;
    opacity: 0.9;
  }

  .platform-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1.5rem;
    margin-bottom: 3rem;
  }

  .platform-card {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 1.5rem;
  }

  .platform-card h3 {
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--color-border);
    padding-bottom: 0.5rem;
  }

  .download-link {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: var(--color-background);
    border-radius: 4px;
    text-decoration: none;
    transition: background 0.2s;
  }

  .download-link:hover {
    background: var(--color-primary-light);
  }

  .filename {
    font-family: monospace;
    font-size: 0.9rem;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
  }

  .requirements {
    margin-top: 1rem;
    font-size: 0.9rem;
  }

  .checksums pre {
    background: var(--color-surface);
    padding: 1rem;
    border-radius: 4px;
    overflow-x: auto;
    font-size: 0.85rem;
  }

  .install-tabs details {
    margin-bottom: 1rem;
    background: var(--color-surface);
    border-radius: 4px;
    padding: 1rem;
  }

  .install-tabs summary {
    font-weight: bold;
    cursor: pointer;
  }

  .method {
    margin-bottom: 1.5rem;
  }

  .method pre {
    background: var(--color-surface);
    padding: 1rem;
    border-radius: 4px;
  }

  .release-info {
    text-align: center;
    margin-top: 3rem;
    padding-top: 2rem;
    border-top: 1px solid var(--color-border);
    color: var(--color-text-secondary);
  }

  .loading, .error {
    text-align: center;
    padding: 3rem;
  }

  .spinner {
    display: inline-block;
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
```

### 2. Release API Endpoint

Create `web/src/routes/api/releases/latest/+server.ts`:

```typescript
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

interface ReleaseInfo {
  version: string;
  date: string;
  downloads: {
    macos_arm64: string;
    macos_x64: string;
    windows: string;
    linux_appimage: string;
    linux_deb: string;
  };
  checksums: Record<string, string>;
}

export const GET: RequestHandler = async ({ fetch }) => {
  const GITHUB_REPO = 'tachikoma/tachikoma';
  const CDN_BASE = 'https://releases.tachikoma.dev';

  try {
    // Fetch latest release from GitHub API
    const response = await fetch(
      `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`,
      {
        headers: {
          Accept: 'application/vnd.github.v3+json',
          // Add token if needed for rate limiting
          // Authorization: `token ${process.env.GITHUB_TOKEN}`,
        },
      }
    );

    if (!response.ok) {
      throw new Error(`GitHub API error: ${response.status}`);
    }

    const release = await response.json();
    const version = release.tag_name.replace(/^v/, '');
    const date = new Date(release.published_at).toISOString().split('T')[0];

    // Build download URLs (prefer CDN, fallback to GitHub)
    const getDownloadUrl = (filename: string): string => {
      const asset = release.assets.find((a: any) => a.name === filename);
      if (asset) {
        // Use CDN if available
        return `${CDN_BASE}/releases/v${version}/${filename}`;
      }
      // Fallback to GitHub
      return `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${filename}`;
    };

    // Extract checksums from release body or separate file
    const checksums: Record<string, string> = {};
    const checksumMatch = release.body?.match(/```\s*[\s\S]*?SHA256[\s\S]*?```/);
    if (checksumMatch) {
      const lines = checksumMatch[0].split('\n');
      for (const line of lines) {
        const match = line.match(/^([a-f0-9]{64})\s+(.+)$/);
        if (match) {
          checksums[match[2]] = match[1];
        }
      }
    }

    const releaseInfo: ReleaseInfo = {
      version,
      date,
      downloads: {
        macos_arm64: getDownloadUrl(`Tachikoma-${version}-arm64.dmg`),
        macos_x64: getDownloadUrl(`Tachikoma-${version}-x64.dmg`),
        windows: getDownloadUrl(`Tachikoma-Setup-${version}.exe`),
        linux_appimage: getDownloadUrl(`Tachikoma-${version}.AppImage`),
        linux_deb: getDownloadUrl(`tachikoma_${version}_amd64.deb`),
      },
      checksums,
    };

    return json(releaseInfo, {
      headers: {
        'Cache-Control': 'public, max-age=300', // 5 minute cache
      },
    });
  } catch (error) {
    console.error('Failed to fetch release:', error);

    // Return fallback/cached response
    return json(
      { error: 'Failed to fetch release information' },
      { status: 500 }
    );
  }
};
```

### 3. Static Download Page Generation

Create `scripts/generate-download-page.ts`:

```typescript
#!/usr/bin/env ts-node
/**
 * Generate static download page data for SSG
 */

import * as fs from 'fs';
import { execSync } from 'child_process';

interface Release {
  version: string;
  date: string;
  downloads: Record<string, string>;
  checksums: Record<string, string>;
}

async function fetchLatestRelease(): Promise<Release> {
  const GITHUB_REPO = 'tachikoma/tachikoma';

  const response = await fetch(
    `https://api.github.com/repos/${GITHUB_REPO}/releases/latest`
  );

  const release = await response.json();
  const version = release.tag_name.replace(/^v/, '');

  return {
    version,
    date: new Date(release.published_at).toISOString().split('T')[0],
    downloads: buildDownloadUrls(version),
    checksums: parseChecksums(release.body || ''),
  };
}

function buildDownloadUrls(version: string): Record<string, string> {
  const base = `https://github.com/tachikoma/tachikoma/releases/download/v${version}`;

  return {
    macos_arm64: `${base}/Tachikoma-${version}-arm64.dmg`,
    macos_x64: `${base}/Tachikoma-${version}-x64.dmg`,
    windows: `${base}/Tachikoma-Setup-${version}.exe`,
    linux_appimage: `${base}/Tachikoma-${version}.AppImage`,
    linux_deb: `${base}/tachikoma_${version}_amd64.deb`,
  };
}

function parseChecksums(body: string): Record<string, string> {
  const checksums: Record<string, string> = {};
  const lines = body.split('\n');

  for (const line of lines) {
    const match = line.match(/^([a-f0-9]{64})\s+(.+)$/);
    if (match) {
      checksums[match[2].trim()] = match[1];
    }
  }

  return checksums;
}

async function main() {
  console.log('Fetching latest release information...');

  const release = await fetchLatestRelease();

  const outputPath = 'web/src/lib/data/release.json';
  fs.mkdirSync('web/src/lib/data', { recursive: true });
  fs.writeFileSync(outputPath, JSON.stringify(release, null, 2));

  console.log(`Release data written to ${outputPath}`);
  console.log(`Version: ${release.version}`);
}

main().catch(console.error);
```

### 4. SEO and Meta Tags

Create `web/src/routes/download/+page.ts`:

```typescript
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch }) => {
  const response = await fetch('/api/releases/latest');
  const release = await response.json();

  return {
    release,
    meta: {
      title: 'Download Tachikoma - AI Development Assistant',
      description: `Download Tachikoma ${release.version} for macOS, Windows, or Linux. AI-powered autonomous development for your desktop.`,
      keywords: 'tachikoma, download, ai assistant, development tools, electron app',
    },
  };
};
```

---

## Testing Requirements

1. Platform detection works correctly
2. Download links resolve correctly
3. Checksums display accurately
4. Page loads without release data (fallback)
5. Mobile responsive layout works

---

## Related Specs

- Depends on: [508-distribution-cdn.md](508-distribution-cdn.md)
- Next: [510-build-tests.md](510-build-tests.md)
- Related: [507-release-notes.md](507-release-notes.md)
