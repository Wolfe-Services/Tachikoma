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

  export let data;
  let platform: 'macos' | 'windows' | 'linux' | 'unknown' = 'unknown';
  let arch: 'arm64' | 'x64' = 'x64';
  let release: Release | null = data?.release || null;
  let loading = !release;

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

    // Fetch latest release if not already loaded
    if (!release) {
      try {
        const response = await fetch('/api/releases/latest');
        release = await response.json();
      } catch (e) {
        console.error('Failed to fetch release info:', e);
      }
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
  <title>{data?.meta?.title || 'Download Tachikoma'}</title>
  <meta name="description" content={data?.meta?.description || 'Download Tachikoma - AI-powered development assistant for your desktop'} />
  <meta name="keywords" content={data?.meta?.keywords || 'tachikoma, download, ai assistant, development tools'} />
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
        <p class="detected">Auto-detected: {platform} {arch}</p>
      {:else}
        <p>Select your platform below</p>
      {/if}
    </section>

    <section class="all-downloads">
      <h2>All Downloads</h2>

      <div class="platform-grid">
        <!-- macOS -->
        <div class="platform-card">
          <h3>🍎 macOS</h3>
          <div class="downloads">
            <a href={release.downloads.macos_arm64} class="download-link">
              <span class="filename">Tachikoma-{release.version}-arm64.dmg</span>
              <span class="badge apple-silicon">Apple Silicon</span>
            </a>
            <a href={release.downloads.macos_x64} class="download-link">
              <span class="filename">Tachikoma-{release.version}-x64.dmg</span>
              <span class="badge intel">Intel</span>
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
          <h3>🪟 Windows</h3>
          <div class="downloads">
            <a href={release.downloads.windows} class="download-link">
              <span class="filename">Tachikoma-Setup-{release.version}.exe</span>
              <span class="badge windows">64-bit</span>
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
          <h3>🐧 Linux</h3>
          <div class="downloads">
            <a href={release.downloads.linux_appimage} class="download-link">
              <span class="filename">Tachikoma-{release.version}.AppImage</span>
              <span class="badge universal">Universal</span>
            </a>
            <a href={release.downloads.linux_deb} class="download-link">
              <span class="filename">tachikoma_{release.version}_amd64.deb</span>
              <span class="badge debian">Debian/Ubuntu</span>
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
      <h2>🔒 Verify Your Download</h2>
      <p>Verify the integrity of your download using SHA-256 checksums:</p>

      <div class="checksums">
        <pre>{#each Object.entries(release.checksums) as [file, hash]}
{hash}  {file}
{/each}</pre>
      </div>

      <details class="verify-instructions">
        <summary>How to verify checksums</summary>
        <div class="instructions">
          <h4>macOS / Linux</h4>
          <pre><code>shasum -a 256 Tachikoma-*.dmg</code></pre>
          <pre><code>sha256sum Tachikoma-*.AppImage</code></pre>

          <h4>Windows (PowerShell)</h4>
          <pre><code>Get-FileHash Tachikoma-Setup-*.exe -Algorithm SHA256</code></pre>
        </div>
      </details>
    </section>

    <section class="installation">
      <h2>📋 Installation Instructions</h2>

      <div class="install-tabs">
        <details open>
          <summary>🍎 macOS</summary>
          <ol>
            <li>Download the DMG file for your Mac</li>
            <li>Open the DMG file</li>
            <li>Drag Tachikoma to the Applications folder</li>
            <li>Open Tachikoma from Applications</li>
            <li>If prompted, click "Open" to allow the app from an identified developer</li>
          </ol>
        </details>

        <details>
          <summary>🪟 Windows</summary>
          <ol>
            <li>Download the installer (.exe)</li>
            <li>Run the installer as Administrator</li>
            <li>Follow the installation wizard</li>
            <li>Launch Tachikoma from the Start Menu or Desktop shortcut</li>
          </ol>
        </details>

        <details>
          <summary>🐧 Linux (AppImage)</summary>
          <ol>
            <li>Download the AppImage</li>
            <li>Make it executable: <code>chmod +x Tachikoma-*.AppImage</code></li>
            <li>Run: <code>./Tachikoma-*.AppImage</code></li>
          </ol>
        </details>

        <details>
          <summary>🐧 Linux (Debian/Ubuntu)</summary>
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
      <h2>📦 Alternative Installation Methods</h2>

      <div class="methods-grid">
        <div class="method">
          <h3>Homebrew (macOS)</h3>
          <pre><code>brew install --cask tachikoma</code></pre>
        </div>

        <div class="method">
          <h3>Snap (Linux)</h3>
          <pre><code>snap install tachikoma</code></pre>
        </div>

        <div class="method">
          <h3>Docker</h3>
          <pre><code>docker run -it tachikoma/tachikoma</code></pre>
        </div>
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
      <h2>⚠️ Unable to load release information</h2>
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
    font-size: 3rem;
    margin-bottom: 0.5rem;
    color: var(--text);
  }

  .tagline {
    color: var(--text-muted);
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
    padding: 1.25rem 2.5rem;
    background: var(--accent);
    color: white;
    border-radius: 12px;
    text-decoration: none;
    font-size: 1.1rem;
    transition: all 0.3s ease;
    box-shadow: 0 4px 15px rgba(0, 180, 216, 0.3);
  }

  .download-button:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 20px rgba(0, 180, 216, 0.4);
  }

  .download-button .icon {
    font-size: 1.5rem;
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

  .detected {
    margin-top: 0.5rem;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .all-downloads h2 {
    text-align: center;
    margin-bottom: 2rem;
    color: var(--text);
  }

  .platform-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1.5rem;
    margin-bottom: 3rem;
  }

  .platform-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .platform-card h3 {
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
    font-size: 1.25rem;
  }

  .downloads {
    margin-bottom: 1rem;
  }

  .download-link {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    background: var(--bg);
    border-radius: 8px;
    text-decoration: none;
    transition: background 0.2s;
    color: var(--text);
  }

  .download-link:hover {
    background: var(--border);
  }

  .filename {
    font-family: 'Monaco', 'Courier New', monospace;
    font-size: 0.9rem;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: bold;
  }

  .badge.apple-silicon {
    background: #ff6b6b;
    color: white;
  }

  .badge.intel {
    background: #4ecdc4;
    color: white;
  }

  .badge.windows {
    background: #0078d4;
    color: white;
  }

  .badge.universal {
    background: #feca57;
    color: #2d3436;
  }

  .badge.debian {
    background: #d63031;
    color: white;
  }

  .requirements {
    font-size: 0.9rem;
  }

  .requirements summary {
    cursor: pointer;
    color: var(--accent);
    margin-bottom: 0.5rem;
  }

  .requirements ul {
    padding-left: 1.5rem;
    color: var(--text-muted);
  }

  .verification, .installation, .alternative-methods {
    margin-bottom: 3rem;
  }

  .verification h2, .installation h2, .alternative-methods h2 {
    color: var(--text);
    margin-bottom: 1rem;
  }

  .checksums {
    margin: 1rem 0;
  }

  .checksums pre {
    background: var(--bg-secondary);
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    font-size: 0.8rem;
    border: 1px solid var(--border);
    color: var(--text-muted);
  }

  .verify-instructions {
    margin-top: 1rem;
  }

  .verify-instructions summary {
    cursor: pointer;
    color: var(--accent);
  }

  .instructions {
    margin-top: 1rem;
    padding: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .instructions h4 {
    margin-bottom: 0.5rem;
    color: var(--text);
  }

  .instructions pre {
    background: var(--bg);
    padding: 0.5rem;
    border-radius: 4px;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
    color: var(--accent);
  }

  .install-tabs details {
    margin-bottom: 1rem;
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 1rem;
    border: 1px solid var(--border);
  }

  .install-tabs summary {
    font-weight: bold;
    cursor: pointer;
    color: var(--accent);
    margin-bottom: 0.5rem;
  }

  .install-tabs ol {
    padding-left: 1.5rem;
    color: var(--text);
  }

  .install-tabs li {
    margin-bottom: 0.5rem;
  }

  .install-tabs code {
    background: var(--bg);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-family: 'Monaco', 'Courier New', monospace;
    color: var(--accent);
  }

  .methods-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5rem;
  }

  .method {
    background: var(--bg-secondary);
    padding: 1rem;
    border-radius: 8px;
    border: 1px solid var(--border);
  }

  .method h3 {
    margin-bottom: 0.5rem;
    color: var(--text);
  }

  .method pre {
    background: var(--bg);
    padding: 0.75rem;
    border-radius: 4px;
    font-size: 0.85rem;
    color: var(--accent);
  }

  .release-info {
    text-align: center;
    margin-top: 3rem;
    padding-top: 2rem;
    border-top: 1px solid var(--border);
    color: var(--text-muted);
  }

  .release-info a {
    color: var(--accent);
    text-decoration: none;
  }

  .release-info a:hover {
    text-decoration: underline;
  }

  .loading, .error {
    text-align: center;
    padding: 3rem;
  }

  .error h2 {
    color: var(--text);
    margin-bottom: 1rem;
  }

  .error a {
    color: var(--accent);
    text-decoration: none;
  }

  .error a:hover {
    text-decoration: underline;
  }

  .spinner {
    display: inline-block;
    width: 40px;
    height: 40px;
    border: 3px solid var(--border);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Mobile responsive */
  @media (max-width: 768px) {
    .download-page {
      padding: 1rem;
    }

    .hero h1 {
      font-size: 2rem;
    }

    .tagline {
      font-size: 1rem;
    }

    .download-button {
      padding: 1rem 2rem;
      font-size: 1rem;
    }

    .platform-grid {
      grid-template-columns: 1fr;
      gap: 1rem;
    }

    .methods-grid {
      grid-template-columns: 1fr;
    }
  }
</style>