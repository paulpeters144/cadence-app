# Plan: Implement `openapi-typescript` for Frontend

This plan outlines the steps to integrate `openapi-typescript` into the frontend, enabling automated generation of TypeScript types from the backend's OpenAPI schema.

## Project Overview
The `cadence-app` uses a Rust backend (Axum with `utoipa`) that exports an OpenAPI 3.0 specification. To ensure type safety across the stack, the frontend should consume these types directly rather than maintaining manual interfaces.

## Technical Stack
- **Frontend:** React, TypeScript, Vite, pnpm
- **Backend:** Rust, Axum, `utoipa`
- **Library:** `openapi-typescript` (v7+)

## Phases of Development

### Phase 1: Research & Discovery
- **Schema Source:** The backend currently exports the OpenAPI schema at `http://localhost:3001/api-docs/openapi.json` (as seen in `backend/src/lib.rs`).
- **Dependencies:** Verify that `openapi-typescript` and `typescript` are compatible with the existing frontend environment.
- **Output Location:** Identify a standard location for generated types, e.g., `frontend/src/lib/api-types.ts`.

### Phase 2: Architecture & Design
- **Integration Strategy:** Use `openapi-typescript` to generate a `paths` interface.
- **Mapping:** Create utility types or wrappers to map the generated `components["schemas"]` to the internal frontend types used by `react-query` or context.
- **Automation:** Define a `package.json` script that can be run locally when the backend is running.

### Phase 3: Implementation Steps
1.  **Install Dependencies:**
    - Install `openapi-typescript` as a dev dependency in the `frontend/` directory.
2.  **Add Script to `package.json`:**
    - Add a `generate-types` script:
      ```json
      "generate-types": "openapi-typescript http://localhost:3001/api-docs/openapi.json -o src/lib/api-types.ts"
      ```
3.  **Generate Initial Types:**
    - Ensure the backend is running and execute the script.
4.  **Refactor Existing Types:**
    - Update `frontend/src/lib/types.ts` or relevant components to use the generated types from `api-types.ts`.
    - Example: `type Task = components["schemas"]["TaskResponse"];`
5.  **Refactor API Calls:**
    - Update fetch calls (e.g., in `frontend/src/lib/api.ts`) to use the new types for request/response payloads.

### Phase 4: Testing & Quality Assurance
- **Static Analysis:** Run `pnpm lint` and `pnpm check` to ensure no type errors were introduced during refactoring.
- **Runtime Verification:** Verify that API interactions still function correctly with the backend.
- **CI/CD Integration (Optional):** Consider adding a check to ensure the generated types are up-to-date with the committed backend schema if a static file is available.

## Potential Challenges
- **Backend Availability:** The generation script requires the backend to be running. A fallback strategy could be to save a static `openapi.json` in the backend repository and point the generator to the local file.
- **Schema Gaps:** If the `utoipa` macros in the backend are missing descriptions or specific fields, the generated types may be incomplete or `any`.
- **Breaking Changes:** Backend updates might break frontend types immediately upon generation, which is intentional but requires prompt refactoring.
