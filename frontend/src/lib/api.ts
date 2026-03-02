import type { List, Task } from "./types";

async function delay(ms: number) {
	return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function fetchWithLatency<T>(url: string): Promise<T> {
	const ms = Math.floor(Math.random() * (1000 - 250 + 1)) + 250;
	await delay(ms);
	const response = await fetch(url);
	if (!response.ok) {
		throw new Error(`Failed to fetch ${url}`);
	}
	return response.json();
}

interface DB {
	user: { username: string } | null;
	lists: List[];
}

let mockDB: DB | null = null;

async function getDB(): Promise<DB> {
	if (mockDB) {
		return mockDB;
	}

	// Initialize from mock JSON
	const [user, lists] = await Promise.all([
		fetchWithLatency<{ username: string }>("/mock/user.json"),
		fetchWithLatency<List[]>("/mock/lists.json"),
	]);

	mockDB = { user, lists };
	return mockDB;
}

export const api = {
	async getUser() {
		const db = await getDB();
		return db.user;
	},

	async login(username: string, password?: string) {
		await delay(500);
		const db = await getDB();
		db.user = { username };
		return db.user;
	},

	async logout() {
		await delay(300);
		const db = await getDB();
		db.user = null;
	},

	async getLists() {
		const db = await getDB();
		return db.lists;
	},

	async addList(name: string) {
		await delay(400);
		const db = await getDB();
		const newList: List = { id: crypto.randomUUID(), name, tasks: [] };
		db.lists.push(newList);
		return newList;
	},

	async deleteList(id: string) {
		await delay(400);
		const db = await getDB();
		db.lists = db.lists.filter((l) => l.id !== id);
	},

	async renameList(id: string, name: string) {
		await delay(300);
		const db = await getDB();
		db.lists = db.lists.map((l) => (l.id === id ? { ...l, name } : l));
	},

	async renameTask(listId: string, taskId: string, title: string) {
		await delay(200);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === listId
				? {
						...l,
						tasks: l.tasks.map((t) => (t.id === taskId ? { ...t, title } : t)),
					}
				: l,
		);
	},

	async archiveList(id: string, archived = true) {
		await delay(300);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === id
				? { ...l, archived, archivedAt: archived ? new Date().toISOString() : undefined }
				: l,
		);
	},

	async updateJournal(id: string, journal: string) {
		await delay(300);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === id ? { ...l, journal } : l,
		);
	},

	async addTask(listId: string, title: string) {
		await delay(500);
		const db = await getDB();
		const newTask: Task = {
			id: crypto.randomUUID(),
			title,
			completed: false,
			createdAt: new Date().toISOString(),
		};
		db.lists = db.lists.map((l) =>
			l.id === listId ? { ...l, tasks: [...l.tasks, newTask] } : l,
		);
		return newTask;
	},

	async toggleTask(listId: string, taskId: string) {
		await delay(200);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === listId
				? {
						...l,
						tasks: l.tasks.map((t) =>
							t.id === taskId
								? {
										...t,
										completed: !t.completed,
										completedAt: !t.completed
											? new Date().toISOString()
											: undefined,
									}
								: t,
						),
					}
				: l,
		);
	},

	async deleteTask(listId: string, taskId: string) {
		await delay(300);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === listId
				? { ...l, tasks: l.tasks.filter((t) => t.id !== taskId) }
				: l,
		);
	},

	async updateTaskPoints(
		listId: string,
		taskId: string,
		points?: 0.5 | 1 | 2 | 3 | 5 | 8,
	) {
		await delay(200);
		const db = await getDB();
		db.lists = db.lists.map((l) =>
			l.id === listId
				? {
						...l,
						tasks: l.tasks.map((t) => (t.id === taskId ? { ...t, points } : t)),
					}
				: l,
		);
	},

	async moveTask(fromListId: string, toListId: string, taskId: string) {
		await delay(400);
		const db = await getDB();
		const fromList = db.lists.find((l) => l.id === fromListId);
		const task = fromList?.tasks.find((t) => t.id === taskId);
		if (!fromList || !task || fromListId === toListId) return;

		db.lists = db.lists.map((l) => {
			if (l.id === fromListId) {
				return { ...l, tasks: l.tasks.filter((t) => t.id !== taskId) };
			}
			if (l.id === toListId) {
				return { ...l, tasks: [...l.tasks, task] };
			}
			return l;
		});
	},

	async reorderTasks(listId: string, activeId: string, overId: string) {
		await delay(200);
		const db = await getDB();
		const list = db.lists.find((l) => l.id === listId);
		if (!list) return;

		const oldIndex = list.tasks.findIndex((t) => t.id === activeId);
		const newIndex = list.tasks.findIndex((t) => t.id === overId);
		if (oldIndex === -1 || newIndex === -1) return;

		const newTasks = [...list.tasks];
		const [movedTask] = newTasks.splice(oldIndex, 1);
		newTasks.splice(newIndex, 0, movedTask);

		db.lists = db.lists.map((l) =>
			l.id === listId ? { ...l, tasks: newTasks } : l,
		);
	},

	async reorderLists(activeId: string, overId: string) {
		await delay(200);
		const db = await getDB();
		const oldIndex = db.lists.findIndex((l) => l.id === activeId);
		const newIndex = db.lists.findIndex((l) => l.id === overId);
		if (oldIndex === -1 || newIndex === -1) return;

		const newLists = [...db.lists];
		const [movedList] = newLists.splice(oldIndex, 1);
		newLists.splice(newIndex, 0, movedList);
		db.lists = newLists;
	},

	async duplicateList(id: string, newName: string) {
		await delay(600);
		const db = await getDB();
		const originalList = db.lists.find((l) => l.id === id);
		if (!originalList) throw new Error("List not found");

		const newList: List = {
			id: crypto.randomUUID(),
			name: newName,
			tasks: originalList.tasks.map((t) => ({
				...t,
				id: crypto.randomUUID(),
				createdAt: new Date().toISOString(),
			})),
		};

		// Insert after the original list
		const originalIndex = db.lists.findIndex((l) => l.id === id);
		db.lists.splice(originalIndex + 1, 0, newList);
		return newList;
	},
};
