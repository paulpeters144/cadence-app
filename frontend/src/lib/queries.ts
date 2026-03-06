import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "./api";
import type { List, Task } from "./types";

export const queryKeys = {
	user: ["user"] as const,
	lists: ["lists"] as const,
};

// --- Queries ---

export function useUser() {
	return useQuery({
		queryKey: queryKeys.user,
		queryFn: () => api.getUser(),
	});
}

export function useLists() {
	return useQuery({
		queryKey: queryKeys.lists,
		queryFn: () => api.getLists(),
	});
}

// --- Mutations ---

export function useLoginMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			username,
			password,
		}: {
			username: string;
			password?: string;
		}) => api.login(username, password),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.user });
		},
	});
}

export function useLogoutMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: () => api.logout(),
		onSuccess: () => {
			queryClient.setQueryData(queryKeys.user, null);
			queryClient.invalidateQueries({ queryKey: queryKeys.user });
		},
	});
}

export function useAddListMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: (name: string) => api.addList(name),
		onMutate: async (name: string) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return [
					...old,
					{
						id: `temp-${crypto.randomUUID()}`,
						name,
						tasks: [],
						archived: false,
						position: 0,
					},
				];
			});

			return { previousLists };
		},
		onError: (_err, _newListName, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useDeleteListMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: (id: string) => api.deleteList(id),
		onMutate: async (id: string) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.filter((l) => l.id !== id);
			});

			return { previousLists };
		},
		onError: (_err, _id, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useDuplicateListMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ id, newName }: { id: string; newName: string }) =>
			api.duplicateList(id, newName),
		onMutate: async ({ id, newName }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				const originalList = old.find((l) => l.id === id);
				if (!originalList) return old;

				const newList: List = {
					...originalList,
					id: `temp-${crypto.randomUUID()}`,
					name: newName,
					tasks: originalList.tasks.map((t) => ({
						...t,
						id: `temp-${crypto.randomUUID()}`,
						createdAt: new Date().toISOString(),
						completed: false,
					})),
				};

				const originalIndex = old.findIndex((l) => l.id === id);
				const newLists = [...old];
				newLists.splice(originalIndex + 1, 0, newList);
				return newLists;
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useRenameListMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ id, name }: { id: string; name: string }) =>
			api.renameList(id, name),
		onMutate: async ({ id, name }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => (l.id === id ? { ...l, name } : l));
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useRenameTaskMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			listId,
			taskId,
			title,
		}: {
			listId: string;
			taskId: string;
			title: string;
		}) => api.renameTask(listId, taskId, title),
		onMutate: async ({ listId, taskId, title }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					return {
						...l,
						tasks: l.tasks.map((t) => (t.id === taskId ? { ...t, title } : t)),
					};
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useArchiveListMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ id, archived }: { id: string; archived?: boolean }) =>
			api.archiveList(id, archived),
		onMutate: async ({ id, archived = true }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) =>
					l.id === id
						? {
								...l,
								archived,
								archivedAt: archived ? new Date().toISOString() : undefined,
							}
						: l,
				);
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useUpdateJournalMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ id, journal }: { id: string; journal: string }) =>
			api.updateJournal(id, journal),
		onMutate: async ({ id, journal }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => (l.id === id ? { ...l, journal } : l));
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useReorderTasksMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			listId,
			activeId,
			overId,
		}: {
			listId: string;
			activeId: string;
			overId: string;
		}) => api.reorderTasks(listId, activeId, overId),
		onMutate: async ({ listId, activeId, overId }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					const oldIndex = l.tasks.findIndex((t) => t.id === activeId);
					const newIndex = l.tasks.findIndex((t) => t.id === overId);
					if (oldIndex === -1 || newIndex === -1) return l;

					const newTasks = [...l.tasks];
					const [movedTask] = newTasks.splice(oldIndex, 1);
					newTasks.splice(newIndex, 0, movedTask);
					return { ...l, tasks: newTasks };
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useReorderListsMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ activeId, overId }: { activeId: string; overId: string }) =>
			api.reorderLists(activeId, overId),
		onMutate: async ({ activeId, overId }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				const oldIndex = old.findIndex((l) => l.id === activeId);
				const newIndex = old.findIndex((l) => l.id === overId);
				if (oldIndex === -1 || newIndex === -1) return old;

				const newLists = [...old];
				const [movedList] = newLists.splice(oldIndex, 1);
				newLists.splice(newIndex, 0, movedList);
				return newLists;
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useAddTaskMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ listId, title }: { listId: string; title: string }) =>
			api.addTask(listId, title),
		onMutate: async ({ listId, title }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					const newTask: Task = {
						id: crypto.randomUUID(),
						title,
						completed: false,
						createdAt: new Date().toISOString(),
						position: 0,
					};
					return { ...l, tasks: [...l.tasks, newTask] };
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useToggleTaskMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: async ({ listId, taskId }: { listId: string; taskId: string }) => {
			const lists = queryClient.getQueryData<List[]>(queryKeys.lists);
			const list = lists?.find((l) => l.id === listId);
			const task = list?.tasks.find((t) => t.id === taskId);
			const newCompleted = !task?.completed;
			return api.toggleTask(listId, taskId, newCompleted);
		},
		onMutate: async ({ listId, taskId }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					return {
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
					};
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useDeleteTaskMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({ listId, taskId }: { listId: string; taskId: string }) =>
			api.deleteTask(listId, taskId),
		onMutate: async ({ listId, taskId }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					return {
						...l,
						tasks: l.tasks.filter((t) => t.id !== taskId),
					};
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useUpdateTaskPointsMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			listId,
			taskId,
			points,
		}: {
			listId: string;
			taskId: string;
			points?: 0.5 | 1 | 2 | 3 | 5 | 8;
		}) => api.updateTaskPoints(listId, taskId, points),
		onMutate: async ({ listId, taskId, points }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				return old.map((l) => {
					if (l.id !== listId) return l;
					return {
						...l,
						tasks: l.tasks.map((t) => (t.id === taskId ? { ...t, points } : t)),
					};
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}

export function useMoveTaskMutation() {
	const queryClient = useQueryClient();
	return useMutation({
		mutationFn: ({
			fromListId,
			toListId,
			taskId,
		}: {
			fromListId: string;
			toListId: string;
			taskId: string;
		}) => api.moveTask(fromListId, toListId, taskId),
		onMutate: async ({ fromListId, toListId, taskId }) => {
			await queryClient.cancelQueries({ queryKey: queryKeys.lists });
			const previousLists = queryClient.getQueryData<List[]>(queryKeys.lists);

			queryClient.setQueryData<List[]>(queryKeys.lists, (old) => {
				if (!old) return [];
				const fromList = old.find((l) => l.id === fromListId);
				const task = fromList?.tasks.find((t) => t.id === taskId);
				if (!fromList || !task || fromListId === toListId) return old;

				return old.map((l) => {
					if (l.id === fromListId) {
						return { ...l, tasks: l.tasks.filter((t) => t.id !== taskId) };
					}
					if (l.id === toListId) {
						return { ...l, tasks: [...l.tasks, task] };
					}
					return l;
				});
			});

			return { previousLists };
		},
		onError: (_err, _vars, context) => {
			if (context?.previousLists) {
				queryClient.setQueryData(queryKeys.lists, context.previousLists);
			}
		},
		onSettled: () => {
			queryClient.invalidateQueries({ queryKey: queryKeys.lists });
		},
	});
}
