import type { components } from "./api-types";

export type Task = components["schemas"]["TaskResponse"];

export type List = components["schemas"]["ListResponse"] & {
	tasks: Task[];
};

export type User = components["schemas"]["UserResponse"];
export type LoginRequest = components["schemas"]["LoginRequest"];
export type LoginResponse = components["schemas"]["LoginResponse"];
export type RegisterRequest = components["schemas"]["RegisterRequest"];
export type RegisterResponse = components["schemas"]["RegisterResponse"];
export type CreateListRequest = components["schemas"]["CreateListRequest"];
export type UpdateListRequest = components["schemas"]["UpdateListRequest"];
export type CreateTaskRequest = components["schemas"]["CreateTaskRequest"];
export type UpdateTaskRequest = components["schemas"]["UpdateTaskRequest"];
export type ErrorResponse = components["schemas"]["ErrorResponse"];
