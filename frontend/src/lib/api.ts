import type { List, Task, User, LoginResponse } from "./types";

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3001/api";

async function fetchAPI<T = void>(endpoint: string, options?: RequestInit): Promise<T> {
	const token = localStorage.getItem("access_token");
	const headers: Record<string, string> = {
		"Content-Type": "application/json",
		...(options?.headers as Record<string, string> || {}),
	};

	if (token) {
		headers["Authorization"] = `Bearer ${token}`;
	}

	const response = await fetch(`${API_BASE_URL}${endpoint}`, {
		...options,
		headers,
	});

	if (!response.ok) {
		if (response.status === 401) {
			localStorage.removeItem("access_token");
			window.location.href = "/login"; // redirect on 401
		}
		let errorMessage = `Failed to fetch ${endpoint}`;
		try {
			const errorData = await response.json();
			if (errorData?.message) {
				errorMessage = errorData.message;
			}
		} catch {
			// Ignore if not json
		}
		throw new Error(errorMessage);
	}

	// 204 No Content
	if (response.status === 204) {
		return undefined as T;
	}

	return response.json();
}

export const api = {
	async getUser(): Promise<User | null> {
		const token = localStorage.getItem("access_token");
		if (!token) return null;
		try {
			return await fetchAPI<User>("/user/me");
		} catch (error) {
			console.error("Failed to get user:", error);
			return null;
		}
	},

	async login(username: string, password?: string): Promise<User> {
		// Mock API didn't use real password, but backend requires it. We'll use "password" as a dummy if not provided.
		const response = await fetchAPI<LoginResponse>("/user/login", {
			method: "POST",
			body: JSON.stringify({ username, password: password || "password" }),
		});
		localStorage.setItem("access_token", response.access_token);
		return { username: response.username };
	},

	async logout(): Promise<void> {
		localStorage.removeItem("access_token");
	},

	async getLists(): Promise<List[]> {
		return fetchAPI<List[]>("/lists");
	},

	async addList(name: string): Promise<List> {
		return fetchAPI<List>("/lists", {
			method: "POST",
			body: JSON.stringify({ name }),
		});
	},

	async deleteList(id: string): Promise<void> {
		return fetchAPI(`/lists/${id}`, {
			method: "DELETE",
		});
	},

	async renameList(id: string, name: string): Promise<List> {
		return fetchAPI<List>(`/lists/${id}`, {
			method: "PATCH",
			body: JSON.stringify({ name }),
		});
	},

	async renameTask(listId: string, taskId: string, title: string): Promise<Task> {
		return fetchAPI<Task>(`/lists/${listId}/tasks/${taskId}`, {
			method: "PATCH",
			body: JSON.stringify({ title }),
		});
	},

	async archiveList(id: string, archived = true): Promise<List> {
		return fetchAPI<List>(`/lists/${id}`, {
			method: "PATCH",
			body: JSON.stringify({ archived }),
		});
	},

	async updateJournal(id: string, journal: string): Promise<List> {
		return fetchAPI<List>(`/lists/${id}`, {
			method: "PATCH",
			body: JSON.stringify({ journal }),
		});
	},

	async addTask(listId: string, title: string): Promise<Task> {
		return fetchAPI<Task>(`/lists/${listId}/tasks`, {
			method: "POST",
			body: JSON.stringify({ title }),
		});
	},

	async toggleTask(listId: string, taskId: string, completed: boolean): Promise<Task> {
		return fetchAPI<Task>(`/lists/${listId}/tasks/${taskId}`, {
			method: "PATCH",
			body: JSON.stringify({ completed }),
		});
	},

	async deleteTask(listId: string, taskId: string): Promise<void> {
		return fetchAPI(`/lists/${listId}/tasks/${taskId}`, {
			method: "DELETE",
		});
	},

	async updateTaskPoints(
		listId: string,
		taskId: string,
		points?: 0.5 | 1 | 2 | 3 | 5 | 8 | null,
	): Promise<Task> {
		return fetchAPI<Task>(`/lists/${listId}/tasks/${taskId}`, {
			method: "PATCH",
			body: JSON.stringify({ points }),
		});
	},

	async moveTask(fromListId: string, toListId: string, taskId: string): Promise<Task> {
		return fetchAPI<Task>(`/tasks/${taskId}/move`, {
			method: "POST",
			body: JSON.stringify({ fromListId, toListId }),
		});
	},

	async reorderTasks(listId: string, activeId: string, overId: string): Promise<Task> {
		return fetchAPI<Task>(`/lists/${listId}/tasks/reorder`, {
			method: "POST",
			body: JSON.stringify({ activeId, overId }),
		});
	},

	async reorderLists(activeId: string, overId: string): Promise<List> {
		return fetchAPI<List>("/lists/reorder", {
			method: "POST",
			body: JSON.stringify({ activeId, overId }),
		});
	},

	async duplicateList(id: string, newName: string): Promise<List> {
		return fetchAPI<List>(`/lists/${id}/duplicate`, {
			method: "POST",
			body: JSON.stringify({ name: newName }),
		});
	},
};
