# Backend API Documentation

This document outlines the required API endpoints and data structures needed to replace the current mock API in the Cadence application.

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

---

## Endpoints

### Auth / User

#### Get Current User
- **URL:** `GET /api/user`
- **Response:** `200 OK`
- **Body:**
  ```json
  {
    "username": "demo_user"
  }
  ```
- **Note:** Returns `null` or `401` if not authenticated. Requires `Authorization: Bearer <access_token>` header.

#### Login
- **URL:** `POST /api/login`
- **Request Body:**
  ```json
  {
    "username": "string",
    "password": "string (optional)"
  }
  ```
- **Response:** `200 OK`
- **Body:** 
  ```json
  {
    "user": {
      "username": "string"
    },
    "access_token": "string",
    "refresh_token": "string"
  }
  ```

#### Refresh Token
- **URL:** `POST /api/refresh`
- **Request Body:**
  ```json
  {
    "refresh_token": "string"
  }
  ```
- **Response:** `200 OK`
- **Body:**
  ```json
  {
    "access_token": "string",
    "refresh_token": "string"
  }
  ```

#### Logout
- **URL:** `POST /api/logout`
- **Response:** `204 No Content`
- **Note:** Should invalidate the refresh token on the backend.

---

### Lists

#### Get All Lists
- **URL:** `GET /api/lists`
- **Response:** `200 OK`
- **Body:** `List[]`
- **Example Response:**
  ```json
  [
    {
      "id": "personal-1",
      "name": "Personal",
      "journal": "Notes about personal goals...",
      "archived": false,
      "tasks": [ ... ]
    }
  ]
  ```

#### Create List
- **URL:** `POST /api/lists`
- **Request Body:**
  ```json
  {
    "name": "string"
  }
  ```
- **Response:** `201 Created`
- **Body:** `List` object (newly created).

#### Update List (Rename / Archive / Journal)
- **URL:** `PATCH /api/lists/:id`
- **Request Body:**
  ```json
  {
    "name": "string (optional)",
    "journal": "string (optional)",
    "archived": "boolean (optional)",
    "archivedAt": "ISO8601 string (optional)"
  }
  ```
- **Response:** `200 OK`
- **Body:** Updated `List` object.

#### Delete List
- **URL:** `DELETE /api/lists/:id`
- **Response:** `204 No Content`

#### Duplicate List
- **URL:** `POST /api/lists/:id/duplicate`
- **Request Body:**
  ```json
  {
    "name": "string"
  }
  ```
- **Response:** `201 Created`
- **Body:** `List` object (newly created duplicate).

#### Reorder Lists
- **URL:** `POST /api/lists/reorder`
- **Request Body:**
  ```json
  {
    "activeId": "uuid",
    "overId": "uuid"
  }
  ```
- **Response:** `200 OK`
- **Note:** Updates the display order of lists for the user.

---

### Tasks

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

#### Update Task (Toggle / Points)
- **URL:** `PATCH /api/lists/:listId/tasks/:taskId`
- **Request Body:**
  ```json
  {
    "completed": "boolean (optional)",
    "completedAt": "ISO8601 string (optional)",
    "points": "number (optional)",
    "title": "string (optional)"
  }
  ```
- **Response:** `200 OK`
- **Body:** Updated `Task` object.

#### Delete Task
- **URL:** `DELETE /api/lists/:listId/tasks/:taskId`
- **Response:** `204 No Content`

#### Move Task to Another List
- **URL:** `POST /api/tasks/:taskId/move`
- **Request Body:**
  ```json
  {
    "fromListId": "uuid",
    "toListId": "uuid"
  }
  ```
- **Response:** `200 OK`

#### Reorder Tasks within a List
- **URL:** `POST /api/lists/:listId/tasks/reorder`
- **Request Body:**
  ```json
  {
    "activeId": "uuid",
    "overId": "uuid"
  }
  ```
- **Response:** `200 OK`

---

## Implementation Notes

1. **IDs:** The frontend currently uses `crypto.randomUUID()` for temporary IDs. The backend should generate permanent UUIDs.
2. **Timestamps:** Use ISO8601 format for all dates (e.g., `2026-03-01T08:50:22Z`).
3. **Persistence:** The current app resets on every page reload (except what's in `mockDB` memory). A real backend must persist these changes to a database.
4. **Auth:** The application should use JWT (JSON Web Tokens) for authentication.
   - Use `access_token` for authenticating API requests via the `Authorization: Bearer <token>` header.
   - Use `refresh_token` to obtain a new `access_token` when it expires.
   - Store tokens securely on the frontend (e.g., HTTP-only cookies or secure storage).
