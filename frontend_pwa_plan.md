# Frontend PWA Integration Plan

This plan outlines the steps to transform the React frontend into a Progressive Web App (PWA) specifically for production builds, while keeping the development environment unaffected.

## Project Overview
Enhance the existing React application with PWA capabilities using `vite-plugin-pwa`. Key features include offline availability, installability on mobile/desktop, and a user-friendly update notification system.

## Technical Stack
- **PWA Tooling**: `vite-plugin-pwa`
- **Caching Engine**: Workbox (Internal to the plugin)
- **Framework**: React 19 (TypeScript)
- **Bundler**: Vite 8

## Phases of Development

### 1. Research & Discovery
- **Dependency Audit**: Verify compatibility with Vite 8.0.0-beta.13.
- **Asset Requirements**: Define necessary assets:
  - `pwa-192x192.png`
  - `pwa-512x512.png`
  - `maskable-icon.png` (for Android)
  - `favicon.svg`
- **Route Handling**: Confirm React Router's compatibility with the service worker's `navigateFallback` configuration.

### 2. Architecture & Design
- **Conditional Activation**: Logic within `vite.config.ts` will ensure the plugin is only active when `mode === 'production'` or during the `build` command.
- **Update Strategy**: Use the "Prompt for update" strategy (`registerType: 'prompt'`) to avoid unexpected page refreshes and give users control.
- **Component Placement**: The PWA controller (`ReloadPrompt`) will be placed at the root of the component tree to listen for service worker events globally.

### 3. Implementation Steps

#### Task 1: Environment Setup
- Install the plugin: `pnpm add -D vite-plugin-pwa`
- Add types to `tsconfig.app.json`:
  ```json
  "compilerOptions": {
    "types": ["vite-plugin-pwa/client"]
  }
  ```

#### Task 2: PWA Assets
- Generate and place PWA icons in `frontend/public/`.
- Create a basic `manifest.json` configuration within the plugin setup.

#### Task 3: Vite Configuration (`frontend/vite.config.ts`)
- Implement conditional plugin inclusion:
  ```typescript
  import { VitePWA } from 'vite-plugin-pwa'
  // ...
  plugins: [
    react(),
    tailwindcss(),
    process.env.NODE_ENV === 'production' && VitePWA({
      registerType: 'prompt',
      includeAssets: ['favicon.svg', 'robots.txt', 'apple-touch-icon.png'],
      manifest: {
        name: 'Cadence App',
        short_name: 'Cadence',
        theme_color: '#ffffff',
        icons: [ /* ... icons ... */ ]
      }
    })
  ].filter(Boolean)
  ```

#### Task 4: Update UI Component
- Create `src/components/ReloadPrompt.tsx`:
  - Use `useRegisterSW()` from `virtual:pwa-register/react`.
  - Display a toast/modal when `needRefresh` is true.
  - Call `updateServiceWorker(true)` when the user confirms.

#### Task 5: Integration
- Mount `<ReloadPrompt />` in `src/App.tsx`.

### 4. Testing & Quality Assurance
- **Dev Isolation**: Verify `pnpm dev` does not generate `sw.js` or register a service worker.
- **Build Verification**: Run `pnpm build && pnpm preview`.
- **Lighthouse Validation**: Use Chrome's Lighthouse tab to run a "PWA" audit.
- **Offline Check**: Disable network in DevTools and confirm the app shell still loads.

## Potential Challenges
- **Vite 8 Beta Hooks**: Monitor for any changes in Vite's build lifecycle that might affect asset discovery.
- **Asset Hashing**: Ensure all static assets are correctly discovered and hashed by Workbox for precise caching.
- **Service Worker Conflicts**: Ensure no existing scripts or legacy service workers interfere with the new registration.
