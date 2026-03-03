# Backend API Documentation

This document outlines the API endpoints and data structures for the Cadence application.

## Data Models

### User
```json
{
  "username": "string"
}
```

### Task
```json
{
  "id": "uuid",
  "title": "string",
  "completed": "boolean",
  "points": "0.5 | 1 | 2 | 3 | 5 | 8 (optional)",
  "createdAt": "ISO8601 string",
  "completedAt": "ISO8601 string (optional)"
}
```

### List
```json
{
  "id": "uuid",
  "name": "string",
  "journal": "string (optional)",
  "archived": "boolean (optional)",
  "archivedAt": "ISO8601 string (optional)",
  "tasks": "Task[]"
}
```

### Error Response
```json
{
  "message": "string"
}
```

---

## Endpoints

### Auth / User

#### Login
- **URL:** `POST /api/user/login`
- **Request Body:**
  ```json
  {
    "username": "string",
    "password": "string"
  }
  ```
- **Response:** `200 OK`
- **Body:** 
  ```json
  {
    "username": "string",
    "access_token": "string"
  }
  ```

#### Register
- **URL:** `POST /api/user/register`
- **Request Body:**
  ```json
  {
    "username": "string",
    "password": "string"
  }
  ```
- **Response:** `200 OK`
- **Body:** 
  ```json
  {
    "username": "string",
    "access_token": "string"
  }
  ```

#### Get Current User (Planned)
- **URL:** `GET /api/user/me`
- **Response:** `200 OK`
- **Body:**
  ```json
  {
    "username": "demo_user"
  }
  ```
- **Note:** Returns `401 Unauthorized` if not authenticated. Requires `Authorization: Bearer <access_token>` header.

#### Logout (Planned)
- **URL:** `POST /api/user/logout`
- **Response:** `204 No Content`
- **Note:** Client should discard the `access_token`.

---

### Lists (Planned)

#### Get All Lists
- **URL:** `GET /api/lists`
- **Response:** `200 OK`
- **Body:** `List[]`

#### Create List
- **URL:** `POST /api/lists`
- **Request Body:**
  ```json
  {
    "name": "string"
  }
  ```
- **Response:** `201 Created`
- **Body:** `List` object.

#### Update List
- **URL:** `PATCH /api/lists/:id`
- **Request Body:**
  ```json
  {
    "name": "string (optional)",
    "journal": "string (optional)",
    "archived": "boolean (optional)"
  }
  ```
- **Response:** `200 OK`
- **Body:** Updated `List` object.

#### Delete List
- **URL:** `DELETE /api/lists/:id`
- **Response:** `204 No Content`

---

### Tasks (Planned)

#### Create Task
- **URL:** `POST /api/lists/:listId/tasks`
- **Request Body:**
  ```json
  {
    "title": "string"
  }
  ```
- **Response:** `201 Created`
- **Body:** `Task` object.

#### Update Task
- **URL:** `PATCH /api/lists/:listId/tasks/:taskId`
- **Request Body:**
  ```json
  {
    "completed": "boolean (optional)",
    "points": "number (optional)",
    "title": "string (optional)"
  }
  ```
- **Response:** `200 OK`
- **Body:** Updated `Task` object.

#### Delete Task
- **URL:** `DELETE /api/lists/:listId/tasks/:taskId`
- **Response:** `204 No Content`

---

## Implementation Notes

1. **IDs:** The backend generates permanent UUIDs for all resources.
2. **Timestamps:** Use ISO8601 format (e.g., `2026-03-01T08:50:22Z`).
3. **Persistence:** Data is persisted in a SQLite database.
4. **Auth:** Simple JWT-based authentication.
   - Use `access_token` for authenticating API requests via the `Authorization: Bearer <token>` header.
   - No refresh tokens are used; users must re-authenticate once the access token expires.
   - Access tokens are signed using `HS256`.
