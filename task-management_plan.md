# Development Plan: Task Management Implementation

This plan outlines the implementation of Task Management for the Cadence backend, enabling users to create, retrieve, and manage tasks within their lists.

## Project Overview
The goal is to implement the "Tasks" section of the `backend_endpoints.md` specification. This includes the data access layer, business logic in the manager, and RESTful API endpoints for:
- Fetching tasks for a specific list.
- Creating new tasks within a list.
- Updating task status (completion, points, title).
- Deleting tasks.

## Technical Stack
- **Language:** Rust
- **Web Framework:** Axum
- **Database:** SQLite (via sqlx)
- **Serialization:** Serde
- **Documentation:** utoipa (OpenAPI/Swagger)
- **Validation:** validator

## Phases of Development

### Phase 1: Research & Discovery
- **Schema Verification:** Confirm `tasks` table constraints in `local_repo.rs` (Foreign Key to `lists`).
- **Authorization Context:** Ensure task operations are scoped to the authenticated user's lists to prevent cross-user data access.
- **Pagination Strategy:** Align `Task` pagination with the existing `List` pagination (`start_id` and `take`).

### Phase 2: Architecture & Design
- **Domain Refinement:** Ensure `Domain::Task` in `domain/list.rs` accurately reflects the database schema and frontend requirements.
- **Repository Pattern:** Define a `TaskRepository` trait to decouple database logic.
- **Error Mapping:** Extend `ManagerError` and `AppError` to handle task-specific failures (e.g., `TaskNotFound`).

### Phase 3: Implementation Steps

#### Task 1: Access Layer (Repository)
- Modify `backend/src/access/local_repo.rs`.
- Define `TaskRepository` trait with methods: `get_tasks`, `create_task`, `update_task`, and `delete_task`.
- Implement `TaskRepository` for `DbUserRepository`.
- **Note:** Ensure SQL queries verify that the `list_id` belongs to the requesting `username`.

#### Task 2: Business Logic (Manager)
- Modify `backend/src/manager/app_manager.rs`.
- Update `Manager` trait to include task management methods.
- Implement methods in `AppManager`, ensuring proper error handling and ownership checks.

#### Task 3: API Handlers
- Create `backend/src/handlers/task.rs`.
- Define request/response schemas (e.g., `CreateTaskRequest`, `UpdateTaskRequest`).
- Implement handlers:
    - `get_tasks` (`GET /api/lists/:listId/tasks`)
    - `create_task` (`POST /api/lists/:listId/tasks`)
    - `update_task` (`PATCH /api/lists/:listId/tasks/:taskId`)
    - `delete_task` (`DELETE /api/lists/:listId/tasks/:taskId`)
- Add `utoipa` annotations for API documentation.

#### Task 4: Routing & Integration
- Modify `backend/src/main.rs`.
- Register new routes in the Axum router.
- Update the `ApiDoc` struct to include the new task handlers for Swagger UI.

### Phase 4: Testing & Quality Assurance
- **Unit Testing:** Add tests in `local_repo.rs` for task CRUD operations.
- **Integration Testing:** Create `backend/tests/task.rs` to verify end-to-end flows:
    - Create a list -> Add tasks -> Fetch tasks -> Update task -> Verify state.
- **Security Testing:** Attempt to access/modify tasks in a list owned by a different user to verify authorization logic.

## Potential Challenges
- **Ownership Verification:** Every task operation must verify the chain: `User -> List -> Task`. A common pitfall is forgetting to check if the `list_id` provided in the URL actually belongs to the authenticated user.
- **SQLite Date Handling:** Ensuring consistent ISO8601 string formatting/parsing between `chrono` and SQLite.
- **Concurrent Updates:** Handling potential race conditions if multiple clients update the same task (though mitigated by SQLite's locking in this scale).
