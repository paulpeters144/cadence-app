# Backend API Documentation

This document outlines the current API endpoints and data structures implemented in the Cadence backend.

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
  "points": "number (optional)",
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
  "archived": "boolean",
  "archivedAt": "ISO8601 string (optional)"
}
```
*Note: `tasks` are not currently returned as part of the list object.*

---

## Endpoints

### Auth / User

#### Get Current User
- **URL:** `GET /api/user/me`
- **Response:** `200 OK`
- **Body:**
  ```json
  {
    "username": "demo_user"
  }
  ```
- **Note:** Requires `Authorization: Bearer <access_token>` header.

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

#### Refresh Token (Coming Soon)
- **URL:** `POST /api/user/refresh`
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

---

### Lists

#### Get All Lists
- **URL:** `GET /api/lists`
- **Query Parameters:**
  - `start_id`: `uuid` (optional) - For pagination
  - `take`: `number` (optional, max 500) - For pagination
- **Response:** `200 OK`
- **Body:** `List[]`
- **Example Response:**
  ```json
  [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Personal",
      "journal": "Notes about personal goals...",
      "archived": false,
      "archivedAt": null
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

#### Duplicate List (Coming Soon)
- **URL:** `POST /api/lists/:id/duplicate`
- **Request Body:**
  ```json
  {
    "name": "string"
  }
  ```
- **Response:** `201 Created`
- **Body:** `List` object (newly created duplicate).

#### Reorder Lists (Coming Soon)
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

#### Get Tasks for a List
- **URL:** `GET /api/lists/:listId/tasks`
- **Query Parameters:**
  - `start_id`: `uuid` (optional) - For pagination
  - `take`: `number` (optional, max 500) - For pagination
- **Response:** `200 OK`
- **Body:** `Task[]`

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

1. **IDs:** The backend generates permanent UUIDs for all objects.
2. **Timestamps:** Use ISO8601 format for all dates (e.g., `2026-03-01T08:50:22Z`).
3. **Persistence:** Data is persisted to a SQLite database.
4. **Auth:** The application uses JWT (JSON Web Tokens) for authentication.
   - Use `access_token` for authenticating API requests via the `Authorization: Bearer <token>` header.
5. **CORS:** Ensure CORS is configured correctly for the frontend to communicate with the backend.
6. **Error Responses:** Standard error response structure:
   ```json
   {
     "error": "Error message description"
   }
   ```

---

## TODO List

### Completed
- [x] User Registration (`POST /api/user/register`)
- [x] User Login (`POST /api/user/login`)
- [x] Get Current User (`GET /api/user/me`)
- [x] Get All Lists (`GET /api/lists`)
- [x] Create List (`POST /api/lists`)
- [x] Update List (`PATCH /api/lists/:id`)
- [x] Delete List (`DELETE /api/lists/:id`)
- [x] Get Tasks for a List (`GET /api/lists/:listId/tasks`)
- [x] Create Task (`POST /api/lists/:listId/tasks`)
- [x] Update Task (`PATCH /api/lists/:listId/tasks/:taskId`)
- [x] Delete Task (`DELETE /api/lists/:listId/tasks/:taskId`)

### Pending
- [ ] Duplicate List (`POST /api/lists/:id/duplicate`)
- [ ] Reorder Lists (`POST /api/lists/reorder`)
- [ ] Move Task to Another List (`POST /api/tasks/:taskId/move`)
- [ ] Reorder Tasks within a List (`POST /api/lists/:listId/tasks/reorder`)
