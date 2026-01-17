/**
 * Cloudflare Worker for serving Tachikoma releases globally
 * Provides geographic distribution for fast downloads
 */

const R2_BUCKET = 'tachikoma-releases';

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;

    // Add CORS headers for API requests
    const corsHeaders = {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, HEAD, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    };

    // Handle OPTIONS requests for CORS
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders });
    }

    // Handle latest version redirect with platform detection
    if (path === '/latest' || path === '/latest/') {
      const platform = url.searchParams.get('platform') || detectPlatform(request);
      const downloadUrl = await getLatestDownloadUrl(env, platform);
      
      if (!downloadUrl) {
        return new Response('No releases available', { 
          status: 404,
          headers: corsHeaders
        });
      }

      return Response.redirect(downloadUrl, 302);
    }

    // Handle direct file requests with geographic optimization
    if (path.startsWith('/releases/')) {
      const key = path.slice(1); // Remove leading /
      
      try {
        const object = await env.R2.get(key);

        if (!object) {
          return new Response('Not found', { 
            status: 404,
            headers: corsHeaders
          });
        }

        const headers = new Headers(corsHeaders);
        headers.set('Content-Type', object.httpMetadata?.contentType || 'application/octet-stream');
        headers.set('Content-Length', object.size);
        headers.set('ETag', object.httpEtag);
        
        // Aggressive caching for versioned releases
        if (path.includes('/releases/v')) {
          headers.set('Cache-Control', 'public, max-age=31536000, immutable'); // 1 year for versioned files
        } else {
          headers.set('Cache-Control', 'public, max-age=300'); // 5 minutes for latest manifests
        }

        // Add CDN-specific headers for performance and bandwidth optimization
        headers.set('CF-Cache-Status', 'HIT'); // Will be overridden by Cloudflare
        headers.set('X-Served-By', 'Tachikoma-CDN');
        
        // Enable compression for appropriate file types
        const filename = path.split('/').pop() || '';
        if (filename.endsWith('.yml') || filename.endsWith('.yaml') || 
            filename.endsWith('.json') || filename.endsWith('.txt')) {
          headers.set('Content-Encoding', 'gzip');
        }
        
        // Add bandwidth optimization headers
        headers.set('Accept-Ranges', 'bytes'); // Enable range requests for partial downloads
        headers.set('X-Content-Type-Options', 'nosniff');
        headers.set('X-Frame-Options', 'DENY');
        
        // Handle range requests for large files (bandwidth optimization)
        const range = request.headers.get('Range');
        if (range && object.size > 10 * 1024 * 1024) { // Only for files > 10MB
          const ranges = parseRangeHeader(range, object.size);
          if (ranges.length === 1) {
            const { start, end } = ranges[0];
            const rangeObject = await env.R2.get(key, {
              range: { offset: start, length: end - start + 1 }
            });
            
            if (rangeObject) {
              headers.set('Content-Range', `bytes ${start}-${end}/${object.size}`);
              headers.set('Content-Length', (end - start + 1).toString());
              return new Response(rangeObject.body, { 
                status: 206,
                headers 
              });
            }
          }
        }

        // Track download metrics
        await trackDownload(env, path, request);

        return new Response(object.body, { headers });

      } catch (error) {
        console.error('Error serving file:', error);
        return new Response('Internal server error', { 
          status: 500,
          headers: corsHeaders
        });
      }
    }

    // Handle update check for auto-updater
    if (path.startsWith('/update/')) {
      return handleUpdateCheck(request, env, path);
    }

    // Handle health check
    if (path === '/health') {
      return new Response(JSON.stringify({
        status: 'ok',
        timestamp: new Date().toISOString(),
        version: '1.0.0'
      }), {
        headers: {
          ...corsHeaders,
          'Content-Type': 'application/json'
        }
      });
    }

    // Handle cost monitoring endpoint
    if (path === '/metrics' || path === '/metrics/') {
      return handleMetrics(request, env);
    }

    // Handle root request - redirect to GitHub
    if (path === '/' || path === '') {
      return Response.redirect('https://github.com/tachikoma-dev/tachikoma', 302);
    }

    return new Response('Not found', { 
      status: 404,
      headers: corsHeaders
    });
  }
};

/**
 * Detect platform from User-Agent header
 */
function detectPlatform(request) {
  const ua = request.headers.get('User-Agent') || '';
  
  // Check for macOS
  if (ua.includes('Mac') || ua.includes('Darwin')) {
    return 'darwin';
  }
  
  // Check for Windows
  if (ua.includes('Windows') || ua.includes('Win32') || ua.includes('Win64')) {
    return 'win32';
  }
  
  // Default to Linux
  return 'linux';
}

/**
 * Get the latest download URL for a platform
 */
async function getLatestDownloadUrl(env, platform) {
  const baseUrl = 'https://releases.tachikoma.dev';

  try {
    // Read latest version from the appropriate manifest
    const manifestFile = platform === 'darwin' ? 'latest-mac.yml' :
                         platform === 'win32' ? 'latest.yml' : 
                         'latest-linux.yml';

    const manifest = await env.R2.get(`releases/latest/${manifestFile}`);
    if (!manifest) {
      console.warn(`No manifest found: ${manifestFile}`);
      return null;
    }

    const content = await manifest.text();
    const versionMatch = content.match(/version:\s*(.+)/);
    if (!versionMatch) {
      console.warn('No version found in manifest');
      return null;
    }

    const version = versionMatch[1].trim();

    // Platform-specific file patterns
    const filePatterns = {
      darwin: [
        `Tachikoma-${version}-arm64.dmg`,
        `Tachikoma-${version}-x64.dmg`,
        `Tachikoma-${version}.dmg`
      ],
      win32: [
        `Tachikoma-Setup-${version}.exe`,
        `Tachikoma-${version}-Setup.exe`
      ],
      linux: [
        `Tachikoma-${version}.AppImage`,
        `Tachikoma-${version}-x86_64.AppImage`
      ],
    };

    const patterns = filePatterns[platform] || filePatterns.linux;
    
    // Try to find the first available file
    for (const filename of patterns) {
      const fileExists = await env.R2.get(`releases/v${version}/${filename}`);
      if (fileExists) {
        return `${baseUrl}/releases/v${version}/${filename}`;
      }
    }

    console.warn(`No files found for platform ${platform}, version ${version}`);
    return null;

  } catch (error) {
    console.error('Error getting latest download URL:', error);
    return null;
  }
}

/**
 * Handle update check requests from electron-updater
 */
async function handleUpdateCheck(request, env, path) {
  // Path format: /update/{platform}/{arch}/{version}
  const parts = path.split('/').filter(Boolean);
  const [, platform, arch, version] = parts;

  const manifestFile = platform === 'darwin' ? 'latest-mac.yml' :
                       platform === 'linux' ? 'latest-linux.yml' : 'latest.yml';

  try {
    const manifest = await env.R2.get(`releases/latest/${manifestFile}`);

    if (!manifest) {
      return new Response('No updates available', { status: 404 });
    }

    const headers = new Headers();
    headers.set('Content-Type', 'text/yaml');
    headers.set('Cache-Control', 'public, max-age=300'); // 5 min cache for update checks

    return new Response(manifest.body, { headers });

  } catch (error) {
    console.error('Error handling update check:', error);
    return new Response('Internal server error', { status: 500 });
  }
}

/**
 * Track download metrics (optional analytics)
 */
async function trackDownload(env, path, request) {
  try {
    const data = {
      path,
      timestamp: Date.now(),
      country: request.cf?.country,
      region: request.cf?.region,
      city: request.cf?.city,
      userAgent: request.headers.get('User-Agent'),
      referer: request.headers.get('Referer'),
      ip: request.headers.get('CF-Connecting-IP'),
    };

    // Track bandwidth usage for cost monitoring
    const contentLength = parseInt(request.headers.get('Content-Length') || '0');
    if (contentLength > 0) {
      data.bytesTransferred = contentLength;
      
      // Store monthly bandwidth usage in KV for cost monitoring
      const monthKey = `bandwidth-${new Date().getFullYear()}-${new Date().getMonth() + 1}`;
      const currentUsage = await env.ANALYTICS?.get(monthKey) || '0';
      const newUsage = parseInt(currentUsage) + contentLength;
      await env.ANALYTICS?.put(monthKey, newUsage.toString());
    }

    // Could send to analytics service like Cloudflare Analytics
    // For now, just log to console
    console.log('Download tracked:', JSON.stringify(data));

    // Store individual download records
    await env.ANALYTICS?.put(`download-${Date.now()}-${Math.random()}`, JSON.stringify(data));

  } catch (error) {
    // Don't fail the request if tracking fails
    console.error('Failed to track download:', error);
  }
}

/**
 * Handle metrics and cost monitoring endpoint
 */
async function handleMetrics(request, env) {
  try {
    // Simple authentication check
    const authHeader = request.headers.get('Authorization');
    const expectedAuth = env.METRICS_AUTH_TOKEN || 'Bearer metrics-token-123';
    
    if (!authHeader || authHeader !== expectedAuth) {
      return new Response('Unauthorized', { status: 401 });
    }

    const now = new Date();
    const currentMonth = `${now.getFullYear()}-${now.getMonth() + 1}`;
    const lastMonth = `${now.getFullYear()}-${now.getMonth()}`;

    // Get bandwidth usage
    const currentBandwidth = await env.ANALYTICS?.get(`bandwidth-${currentMonth}`) || '0';
    const lastMonthBandwidth = await env.ANALYTICS?.get(`bandwidth-${lastMonth}`) || '0';

    // Calculate estimated costs (Cloudflare R2 pricing as of 2024)
    const currentGB = parseInt(currentBandwidth) / (1024 * 1024 * 1024);
    const lastMonthGB = parseInt(lastMonthBandwidth) / (1024 * 1024 * 1024);

    // R2 costs: $0.36/month per million Class A operations (PUT, LIST, DELETE)
    // $0.18/month per million Class B operations (GET, HEAD)
    // No egress charges when accessed via Cloudflare CDN
    const estimatedMonthlyCost = currentGB * 0.36; // Very rough estimate

    const metrics = {
      timestamp: now.toISOString(),
      bandwidth: {
        currentMonth: {
          bytes: parseInt(currentBandwidth),
          gb: currentGB.toFixed(3),
          formatted: formatBytes(parseInt(currentBandwidth))
        },
        lastMonth: {
          bytes: parseInt(lastMonthBandwidth),
          gb: lastMonthGB.toFixed(3),
          formatted: formatBytes(parseInt(lastMonthBandwidth))
        }
      },
      costs: {
        currentMonthEstimate: `$${estimatedMonthlyCost.toFixed(2)}`,
        currency: 'USD',
        note: 'Estimate based on R2 storage costs. Actual costs may vary.'
      },
      limits: {
        monthlyBandwidthLimit: '1TB', // Set your own limits
        currentUsagePercent: (currentGB / 1024 * 100).toFixed(2) + '%'
      }
    };

    return new Response(JSON.stringify(metrics, null, 2), {
      headers: {
        'Content-Type': 'application/json',
        'Cache-Control': 'no-cache'
      }
    });

  } catch (error) {
    console.error('Error handling metrics:', error);
    return new Response('Internal server error', { status: 500 });
  }
}

/**
 * Format bytes into human readable format
 */
function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Parse HTTP Range header for partial content requests
 * @param {string} rangeHeader - The Range header value
 * @param {number} fileSize - Total file size
 * @returns {Array} Array of range objects {start, end}
 */
function parseRangeHeader(rangeHeader, fileSize) {
  const ranges = [];
  
  if (!rangeHeader || !rangeHeader.startsWith('bytes=')) {
    return ranges;
  }
  
  const rangeSpec = rangeHeader.substring(6); // Remove 'bytes=' prefix
  const rangeParts = rangeSpec.split(',');
  
  for (const part of rangeParts) {
    const range = part.trim();
    let start, end;
    
    if (range.startsWith('-')) {
      // Suffix-byte-range-spec: -500 (last 500 bytes)
      const suffixLength = parseInt(range.substring(1));
      start = Math.max(0, fileSize - suffixLength);
      end = fileSize - 1;
    } else if (range.endsWith('-')) {
      // Range from start to end: 200-
      start = parseInt(range.substring(0, range.length - 1));
      end = fileSize - 1;
    } else {
      // Complete range: 200-300
      const dashIndex = range.indexOf('-');
      if (dashIndex > 0) {
        start = parseInt(range.substring(0, dashIndex));
        end = parseInt(range.substring(dashIndex + 1));
      } else {
        continue; // Invalid range format
      }
    }
    
    // Validate range
    if (start >= fileSize || end >= fileSize || start > end) {
      continue; // Invalid range
    }
    
    ranges.push({ start, end });
  }
  
  return ranges;
}