# Migration Plan: Zustand to React Router Data APIs

This plan outlines the steps to remove Zustand and transition the application to use React Router (v7) for data management, fetching from mock JSON files with simulated latency.

## 1. Prepare Mock Data
Create static JSON files in the `public/mock/` directory to serve as the initial data source.
- `public/mock/user.json`: Mock user profile.
- `public/mock/lists.json`: Mock list of task collections with nested tasks.

## 2. Implement API Utility
Create `src/lib/api.ts` to handle data fetching with simulated latency.
- Implement `fetchWithLatency(url: string)`:
  - Generates a random delay between 250ms and 1000ms.
  - Returns the result of a standard `fetch` call after the delay.
- Implement a mock "database" helper using `localStorage` to allow mutations (actions) to persist during the session, initializing from JSON files if `localStorage` is empty.

## 3. Transform Routing Structure
Refactor `src/App.tsx` and `src/main.tsx` to use the Data API.
- Switch from `BrowserRouter` to `createBrowserRouter`.
- Define Loaders:
  - `rootLoader`: Fetches user and lists for `Layout`.
  - `tasksLoader`: Handles identifying the active list.
  - `dashboardLoader`: Prepares stats for the dashboard.
- Define Actions:
  - `authAction`: Handles login/logout.
  - `listAction`: Handles creating, deleting, and renaming lists.
  - `taskAction`: Handles all task-related mutations (add, toggle, delete, points, move, reorder).

## 4. Refactor Components
Update components to consume data from React Router hooks instead of Zustand.
- **Layout.tsx**: 
  - Replace `useStore` with `useLoaderData` (for lists/user) and `useSubmit`/`useFetcher` (for actions).
  - Update DND handlers to trigger router actions.
- **Tasks.tsx**:
  - Replace `useStore` with `useLoaderData` and `useFetcher`.
  - Convert the task form to use React Router's `Form` component or `useFetcher`.
- **Dashboard.tsx**:
  - Replace `useStore` with `useLoaderData`.
  - Remove the `useEffect` that seeds dummy data, as this will now be handled by the mock API initialization.
- **Login.tsx**:
  - Replace `useStore` and `useNavigate` with a standard `Form` and a router action.

## 5. Clean Up
- Delete `src/lib/store.ts`.
- Uninstall `zustand` dependency from `package.json`.
- Verify that all imports of `useStore` are removed.

## 6. Verification
- Ensure all pages load with the simulated latency.
- Verify that mutations (e.g., adding a task) correctly trigger re-validation of loaders.
- Confirm that the application remains functional without the Zustand store.
