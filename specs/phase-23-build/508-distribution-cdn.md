# 508 - Distribution CDN

**Phase:** 23 - Build & Distribution
**Spec ID:** 508
**Status:** Planned
**Dependencies:** 502-auto-update-server
**Estimated Context:** ~6% of Sonnet window

---

## Objective

Configure content delivery network (CDN) infrastructure for hosting and distributing release artifacts globally with high availability and fast download speeds.

---

## Acceptance Criteria

- [ ] CDN configured for release hosting
- [ ] Geographic distribution for fast downloads
- [ ] SSL/TLS for secure downloads
- [ ] Bandwidth optimization
- [ ] Cost monitoring
- [ ] Fallback to GitHub Releases

---

## Implementation Details

### 1. CDN Options

#### Option A: GitHub Releases (Default)
- Pros: Free, integrated with workflow, no additional setup
- Cons: Rate limits, less control

#### Option B: Cloudflare R2 + Workers
- Pros: Generous free tier, global CDN, no egress fees
- Cons: Requires setup

#### Option C: AWS CloudFront + S3
- Pros: Industry standard, highly configurable
- Cons: Egress costs

### 2. Cloudflare R2 Setup

Create `scripts/upload-to-r2.sh`:

```bash
#!/usr/bin/env bash
set -euo pipefail

VERSION="${1:?Version required}"
BUCKET="${R2_BUCKET:-tachikoma-releases}"

# Required environment variables
: "${R2_ACCESS_KEY_ID:?R2_ACCESS_KEY_ID required}"
: "${R2_SECRET_ACCESS_KEY:?R2_SECRET_ACCESS_KEY required}"
: "${R2_ENDPOINT:?R2_ENDPOINT required}"

echo "Uploading release v${VERSION} to R2..."

# Configure AWS CLI for R2
export AWS_ACCESS_KEY_ID="$R2_ACCESS_KEY_ID"
export AWS_SECRET_ACCESS_KEY="$R2_SECRET_ACCESS_KEY"

# Upload release artifacts
for file in electron/out/*; do
    if [ -f "$file" ]; then
        filename=$(basename "$file")
        echo "Uploading: $filename"

        aws s3 cp "$file" "s3://${BUCKET}/releases/v${VERSION}/${filename}" \
            --endpoint-url "$R2_ENDPOINT" \
            --content-type "application/octet-stream"
    fi
done

# Upload latest.yml files for auto-update
for yml in electron/out/*.yml; do
    if [ -f "$yml" ]; then
        filename=$(basename "$yml")
        echo "Uploading: $filename"

        aws s3 cp "$yml" "s3://${BUCKET}/releases/latest/${filename}" \
            --endpoint-url "$R2_ENDPOINT" \
            --content-type "text/yaml"
    fi
done

echo "Upload complete!"
echo "Download URL: https://releases.tachikoma.dev/releases/v${VERSION}/"
```

### 3. Cloudflare Worker for Downloads

Create `cdn/worker.js`:

```javascript
/**
 * Cloudflare Worker for serving Tachikoma releases
 */

const R2_BUCKET = 'tachikoma-releases';

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    // Handle latest version redirect
    if (path === '/latest' || path === '/latest/') {
      const platform = url.searchParams.get('platform') || detectPlatform(request);
      return Response.redirect(await getLatestDownloadUrl(env, platform), 302);
    }

    // Handle direct file requests
    if (path.startsWith('/releases/')) {
      const key = path.slice(1); // Remove leading /
      const object = await env.R2.get(key);

      if (!object) {
        return new Response('Not found', { status: 404 });
      }

      const headers = new Headers();
      headers.set('Content-Type', object.httpMetadata?.contentType || 'application/octet-stream');
      headers.set('Content-Length', object.size);
      headers.set('Cache-Control', 'public, max-age=31536000'); // 1 year for versioned files
      headers.set('ETag', object.httpEtag);

      // Track download
      await trackDownload(env, path, request);

      return new Response(object.body, { headers });
    }

    // Handle update check
    if (path.startsWith('/update/')) {
      return handleUpdateCheck(request, env, path);
    }

    return new Response('Not found', { status: 404 });
  }
};

function detectPlatform(request) {
  const ua = request.headers.get('User-Agent') || '';
  if (ua.includes('Mac')) return 'darwin';
  if (ua.includes('Windows')) return 'win32';
  return 'linux';
}

async function getLatestDownloadUrl(env, platform) {
  const baseUrl = 'https://releases.tachikoma.dev';

  // Read latest version from manifest
  const manifest = await env.R2.get('releases/latest/latest.yml');
  if (!manifest) {
    return `${baseUrl}/releases/`;
  }

  const content = await manifest.text();
  const versionMatch = content.match(/version: (.+)/);
  const version = versionMatch ? versionMatch[1].trim() : 'latest';

  // Platform-specific files
  const files = {
    darwin: `Tachikoma-${version}-arm64.dmg`,
    win32: `Tachikoma-Setup-${version}.exe`,
    linux: `Tachikoma-${version}.AppImage`,
  };

  return `${baseUrl}/releases/v${version}/${files[platform] || files.linux}`;
}

async function handleUpdateCheck(request, env, path) {
  // Path format: /update/{platform}/{arch}/{version}
  const parts = path.split('/').filter(Boolean);
  const [, platform, arch, version] = parts;

  const manifestFile = platform === 'darwin' ? 'latest-mac.yml' :
                       platform === 'linux' ? 'latest-linux.yml' : 'latest.yml';

  const manifest = await env.R2.get(`releases/latest/${manifestFile}`);

  if (!manifest) {
    return new Response('No updates available', { status: 404 });
  }

  const headers = new Headers();
  headers.set('Content-Type', 'text/yaml');
  headers.set('Cache-Control', 'public, max-age=300'); // 5 min cache for update checks

  return new Response(manifest.body, { headers });
}

async function trackDownload(env, path, request) {
  // Track in Analytics (optional)
  const data = {
    path,
    timestamp: Date.now(),
    country: request.cf?.country,
    userAgent: request.headers.get('User-Agent'),
  };

  // Could send to analytics service
  // await env.ANALYTICS.writeDataPoint(data);
}
```

### 4. CDN Configuration in electron-builder

Update `electron/electron-builder.config.js`:

```javascript
publish: [
  // Primary: GitHub Releases
  {
    provider: 'github',
    owner: 'tachikoma',
    repo: 'tachikoma',
  },
  // Secondary: Custom CDN
  {
    provider: 'generic',
    url: 'https://releases.tachikoma.dev/releases/latest',
    channel: 'latest',
  },
],
```

### 5. CI Upload to CDN

Add to `.github/workflows/release.yml`:

```yaml
upload-to-cdn:
  needs: [build-and-release]
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Download artifacts
      uses: actions/download-artifact@v4
      with:
        path: release-artifacts

    - name: Upload to R2
      env:
        R2_ACCESS_KEY_ID: ${{ secrets.R2_ACCESS_KEY_ID }}
        R2_SECRET_ACCESS_KEY: ${{ secrets.R2_SECRET_ACCESS_KEY }}
        R2_ENDPOINT: ${{ secrets.R2_ENDPOINT }}
      run: |
        VERSION=${GITHUB_REF#refs/tags/v}
        ./scripts/upload-to-r2.sh "$VERSION"
```

---

## Testing Requirements

1. Downloads work from CDN URL
2. Auto-update finds CDN manifest
3. Fallback to GitHub works
4. Global latency is acceptable
5. Cost monitoring is in place

---

## Related Specs

- Depends on: [502-auto-update-server.md](502-auto-update-server.md)
- Next: [509-download-page.md](509-download-page.md)
- Related: [503-auto-update-client.md](503-auto-update-client.md)
