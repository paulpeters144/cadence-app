export interface Task {
	id: string;
	title: string;
	completed: boolean;
	points?: 0.5 | 1 | 2 | 3 | 5 | 8;
	createdAt: string;
	completedAt?: string;
}

export interface List {
	id: string;
	name: string;
	tasks: Task[];
	journal?: string;
	archived?: boolean;
	archivedAt?: string;
}
