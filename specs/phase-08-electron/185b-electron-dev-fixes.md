# Spec 185b: Electron Development Mode Fixes

## Header
- **Spec ID**: 185b
- **Phase**: 08 - Electron Shell
- **Component**: Dev Mode Fixes
- **Dependencies**: 161, 169, 170
- **Status**: CRITICAL - Dev mode was broken
- **Priority**: P0 - Blocking development

## Objective
Fix critical bugs in Electron development mode that prevented the app from loading properly.

## Problems Fixed

### 1. Dev Server URL Not Loading
The Electron main process required `ELECTRON_RENDERER_URL` env var to be set, but electron-vite doesn't set it when renderer config is missing.

**Fix Applied**: Default to `http://127.0.0.1:1420` in dev mode.

```typescript
// electron/main/index.ts
if (is.dev) {
  const devServerUrl = process.env['ELECTRON_RENDERER_URL'] || 'http://127.0.0.1:1420';
  await this.mainWindow.loadURL(devServerUrl);
} else {
  await this.mainWindow.loadFile(join(__dirname, '../../web/dist/index.html'));
}
```

### 2. Content Security Policy Blocking Inline Scripts
SvelteKit uses inline scripts for bootstrapping, but CSP had `script-src 'self'` only.

**Fix Applied**: Allow unsafe-inline and unsafe-eval in dev mode.

```html
<!-- web/src/app.html -->
<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self' ws: wss: http: https:;" />
```

### 3. Protocol URL Pattern Error
`webRequest.onBeforeRequest` was using URL patterns for custom schemes (`tachikoma://*/*`) which Electron doesn't support.

**Fix Applied**: Remove URL filter and check in callback instead.

```typescript
// electron/main/protocol/session.ts
ses.webRequest.onBeforeRequest((details, callback) => {
  if (details.url.startsWith('tachikoma:') || details.url.startsWith('tachikoma-asset:')) {
    logger.debug('Custom protocol request', { url: details.url });
  }
  callback({});
});
```

### 4. crypto is not defined (Still needs fix)
The mission:start handler uses `crypto.randomUUID()` but crypto is not available in the main process.

**TODO**: Use Node's crypto module or uuid package.

## Acceptance Criteria
- [x] Dev server URL defaults to 1420 in dev mode
- [x] CSP allows SvelteKit inline scripts
- [x] Protocol session handlers don't crash on custom schemes
- [ ] crypto.randomUUID replaced with proper UUID generation

## Testing
1. Run `npm run dev` from project root
2. Verify Electron window opens and loads from dev server
3. Verify no CSP errors in console
4. Verify no protocol scheme errors
