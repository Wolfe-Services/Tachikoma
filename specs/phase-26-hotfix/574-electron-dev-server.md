# 574 - Electron Dev Server Configuration

**Phase:** 26 - Hotfix
**Spec ID:** 574
**Status:** Planned
**Priority:** P0
**Dependencies:** 552

## Objective

Fix the Electron development server to work seamlessly with the Vite dev server.

## Acceptance Criteria

- [x] Electron uses correct dev server port (5173 for web, 1420 for Tauri)
- [x] ELECTRON_RENDERER_URL defaults to http://localhost:5173 in dev
- [x] Fix crypto undefined error in native.ts (use require crypto)
- [x] electron-vite handles renderer config properly
- [x] Dev server starts both web and electron with single command

## Implementation

1. Update electron/main/index.ts:
   - Change default URL to http://localhost:5173
   
2. Fix crypto in electron/main/native.ts:
   ```typescript
   import { randomUUID } from "crypto";
   ```

3. Add concurrent dev script to package.json:
   ```json
   "dev:all": "concurrently \"npm run dev --prefix ../web\" \"npm run dev\""
   ```
