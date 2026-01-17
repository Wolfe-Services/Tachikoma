# Cloudflare Worker Configuration for Tachikoma CDN

This directory contains the Cloudflare Worker configuration for serving Tachikoma releases with SSL/TLS encryption and geographic distribution.

## Setup

### 1. Create Cloudflare R2 Bucket

```bash
# Install Wrangler CLI
npm install -g @cloudflare/wrangler

# Login to Cloudflare
wrangler login

# Create R2 bucket
wrangler r2 bucket create tachikoma-releases
```

### 2. Configure Domain and SSL

1. Add custom domain to Cloudflare Worker:
   - Domain: `releases.tachikoma.dev`
   - SSL/TLS mode: Full (strict)
   - Always Use HTTPS: On

2. Configure DNS:
   ```
   releases.tachikoma.dev -> CNAME to your-worker.your-subdomain.workers.dev
   ```

### 3. Deploy Worker

```bash
# Install dependencies
npm install

# Deploy to Cloudflare
wrangler deploy worker.js --name tachikoma-cdn
```

### 4. Environment Variables

Configure these secrets in Cloudflare Workers:

```bash
# R2 credentials (for internal worker access)
wrangler secret put R2_ACCESS_KEY_ID
wrangler secret put R2_SECRET_ACCESS_KEY

# Analytics (optional)
wrangler secret put ANALYTICS_TOKEN
```

## SSL/TLS Configuration

The worker automatically provides SSL/TLS through Cloudflare's edge network:

- **TLS 1.3** support for modern clients
- **HTTP/2** and **HTTP/3** support for faster downloads
- **Certificate transparency** logging
- **HSTS** headers for security
- **Automatic certificate renewal**

All downloads are served over HTTPS with end-to-end encryption.

## Performance Features

- **Global CDN**: Files cached at 200+ edge locations
- **Smart routing**: Requests routed to nearest data center
- **Bandwidth optimization**: Automatic compression and optimization
- **Cache control**: Aggressive caching for release files
- **Delta updates**: Supports electron-updater's differential updates

## Monitoring

The worker includes built-in monitoring:

- Request metrics via Cloudflare Analytics
- Error tracking and alerting
- Geographic download distribution
- Bandwidth usage monitoring
- Cost tracking per region

## Security

- HTTPS-only (HTTP redirects to HTTPS)
- CORS configured for web requests
- No directory listing
- Rate limiting (via Cloudflare)
- DDoS protection (via Cloudflare)

## API Endpoints

- `GET /latest?platform=darwin|win32|linux` - Latest release redirect
- `GET /releases/v{version}/{file}` - Direct file download
- `GET /update/{platform}/{arch}/{version}` - Update manifest for electron-updater
- `GET /health` - Health check endpoint