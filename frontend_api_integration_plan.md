# Frontend API Integration Plan

## Project Overview
This plan outlines the steps to migrate the frontend of the **Cadence App** from using local, in-memory mocked data to communicating with the real Rust/Axum backend API. It also identifies missing backend functionality required to fully support the frontend's current data model and user experience.

## Technical Stack
- **Frontend:** React, TypeScript, Vite, TanStack Query (`@tanstack/react-query`)
- **Backend:** Rust, Axum, Serde, Utoipa (OpenAPI)
- **Communication:** RESTful API with JWT Authentication (Bearer Tokens)

---

## Phases of Development

### Phase 1: Research & Discovery (Backend Gaps)
Before modifying the frontend, we must address critical discrepancies between the frontend's expected data shape (from the mock) and the backend's current implementation.

**Missing Backend Functionality:**
1. **Aggregated Lists & Tasks:** The frontend's `List` type includes a nested array of `tasks: Task[]`. Currently, the backend's `GET /api/lists` returns only list metadata (`ListResponse`), meaning tasks must be fetched separately per list (`GET /api/lists/{listId}/tasks`). This creates an N+1 query problem on initial dashboard load.
   - **Action:** Update the backend's `GET /api/lists` to return tasks embedded within each list, OR introduce an aggregated endpoint (e.g., `?include_tasks=true`). This will match the frontend's expected type and avoid massive client-side refactoring and performance issues.
2. **Logout Endpoint (Optional but Recommended):** The backend only has `/api/user/login` and `/api/user/register`. While JWTs can simply be dropped client-side to "logout", adding a backend mechanism to blacklist tokens or clear HTTP-only cookies (if moving away from localStorage) is a best practice.
3. **CORS Configuration:** Ensure the backend Axum server has CORS properly configured (e.g., using `tower_http::cors::CorsLayer`) to accept requests from the Vite dev server (`http://localhost:5173`).

---

### Phase 2: Architecture & Design
- **API Client:** We will refactor `frontend/src/lib/api.ts`. The in-memory `mockDB` and randomized `fetchWithLatency` will be removed entirely.
- **Authentication Strategy:** The `login` endpoint returns an `access_token`. The frontend will store this token in `localStorage` (or memory/sessionStorage) and attach it to the `Authorization: Bearer <token>` header for all subsequent API requests.
- **Environment Variables:** Utilize Vite's `.env` to define a `VITE_API_BASE_URL` (defaulting to `http://localhost:8080/api` locally).
- **Error Handling:** Centralize response parsing to catch `401 Unauthorized` errors globally, which should trigger a client-side logout and redirect the user to the login page.

---

### Phase 3: Implementation Steps

**Step 1: Backend Updates**
1. Modify `Domain::List` and `ListResponse` (or create a new struct `ListWithTasksResponse`) in the backend to include `tasks: Vec<TaskResponse>`.
2. Update the `get_lists` handler in `backend/src/handlers/list.rs` to fetch and embed tasks.
3. Verify CORS is enabled in `backend/src/main.rs` or `lib.rs`.

**Step 2: Frontend API Client Refactor (`frontend/src/lib/api.ts`)**
1. Create a helper function `fetchAPI(endpoint: string, options?: RequestInit)` that automatically reads the JWT from `localStorage` and appends it to headers.
2. Implement global error handling within `fetchAPI` to catch non-2xx statuses (especially 401s).
3. Replace all mock functions in `api.ts` with real `fetch` calls mapping to the Axum routes:
   - `getUser`: `GET /user/me`
   - `login`: `POST /user/login` (must save token to `localStorage` on success)
   - `logout`: Clear token from `localStorage`
   - `getLists`: `GET /lists` (expecting the newly embedded tasks from Step 1)
   - `addList`: `POST /lists`
   - `deleteList`: `DELETE /lists/{id}`
   - `renameList`, `archiveList`, `updateJournal`: `PATCH /lists/{id}`
   - `addTask`: `POST /lists/{listId}/tasks`
   - `toggleTask`, `renameTask`, `updateTaskPoints`: `PATCH /lists/{listId}/tasks/{taskId}`
   - `deleteTask`: `DELETE /lists/{listId}/tasks/{taskId}`
   - `reorderTasks`: `POST /lists/{listId}/tasks/reorder`
   - `reorderLists`: `POST /lists/reorder`
   - `duplicateList`: `POST /lists/{id}/duplicate`
   - `moveTask`: `POST /tasks/{taskId}/move`

**Step 3: React Query Synchronization (`frontend/src/lib/queries.ts`)**
1. The existing optimistic updates in `queries.ts` are robust. However, they rely on the exact data shapes returned by the mutations.
2. Ensure the responses from the real backend mutations (e.g., `addTask` returning the new `TaskResponse`) match what the `onMutate` optimistic updates expect. If there are casing discrepancies (e.g., `createdAt` vs `created_at`), ensure `api-types.ts` (generated from OpenAPI) is respected and mapping is done if needed in `api.ts`.
3. Update cache invalidation logic if any endpoint behavior drastically deviates from the mock.

---

### Phase 4: Testing & Quality Assurance
1. **Unit/Integration Tests:** Update backend unit tests in `backend/tests/` to assert the new `GET /lists` behavior.
2. **End-to-End Flow:** Manually test the full lifecycle on the frontend: User Registration -> Login -> Create List -> Create Task -> Complete Task -> Logout.
3. **Network Tab Auditing:** Verify in browser DevTools that API calls are being made to the actual backend without excessive N+1 queries.
4. **Token Expiry Validation:** Test the `401 Unauthorized` interceptor by manually deleting the token from `localStorage` or letting it expire, ensuring the UI safely redirects to the login screen.

---

### Potential Challenges
- **Optimistic UI Desync:** If the optimistic update applies a change to the UI but the backend rejects the request (e.g., validation failure or constraint violation), the UI will flash back to the old state. TanStack Query handles this gracefully via `onError`, but we must ensure the UX is not jarring (perhaps by showing a toast notification on failure).
- **TypeScript Type Drift:** The frontend relies on generated OpenAPI types. As we alter the backend to support embedded tasks or other changes, we must remember to re-run the OpenAPI export script (`cargo run --bin export_openapi`) and regenerate the frontend types, or the TypeScript compiler will complain.
- **Concurrency & Reordering:** Reordering lists and tasks (`position` updates) concurrently can be tricky. Ensure the backend handles `position` mathematically correctly and that optimistic UI updates don't aggressively overwrite the server state if two clients reorder simultaneously.