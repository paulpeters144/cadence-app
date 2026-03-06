import {
	SortableContext,
	useSortable,
	verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
	Archive,
	Check,
	Circle,
	FileText,
	GripVertical,
	List as ListIcon,
	X,
} from "lucide-react";
import type React from "react";
import { useEffect, useRef, useState } from "react";
import { useSearchParams, useNavigate } from "react-router-dom";
import {
	useArchiveListMutation,
	useUpdateJournalMutation,
	useAddTaskMutation,
	useDeleteTaskMutation,
	useLists,
	useRenameListMutation,
	useRenameTaskMutation,
	useToggleTaskMutation,
	useUpdateTaskPointsMutation,
} from "../lib/queries";
import type { Task } from "../lib/types";
import { cn } from "../lib/utils";

// --- Components ---

function DraggableTask({
	task,
	listId,
	onToggle,
	onDelete,
	onUpdatePoints,
	onRename,
	isArchived,
}: {
	task: Task;
	listId: string;
	onToggle: (lid: string, tid: string, currentCompleted: boolean) => void;
	onDelete: (lid: string, tid: string) => void;
	onUpdatePoints: (lid: string, tid: string, p?: 0.5 | 1 | 2 | 3 | 5 | 8) => void;
	onRename: (lid: string, tid: string, title: string) => void;
	isArchived?: boolean;
}) {
	const {
		attributes,
		listeners,
		setNodeRef,
		transform,
		transition,
		isDragging,
	} = useSortable({
		id: task.id,
		data: { task, listId, type: "task" },
		disabled: isArchived,
	});

	const [isEditing, setIsEditing] = useState(false);
	const [editedTitle, setEditedTitle] = useState(task.title);
	const inputRef = useRef<HTMLInputElement>(null);

	const style = {
		transform: CSS.Translate.toString(transform),
		transition,
		opacity: isDragging ? 0.4 : 1,
	};

	const handleRename = (e?: React.FormEvent) => {
		e?.preventDefault();
		if (editedTitle.trim() && editedTitle !== task.title) {
			onRename(listId, task.id, editedTitle.trim());
		}
		setIsEditing(false);
	};

	return (
		<div
			ref={setNodeRef}
			style={style}
			className={cn(
				"group flex items-center gap-4 py-4 border-b border-border/30 last:border-0 hover:bg-accent/5 px-4 -mx-4 rounded-xl bg-background",
				isDragging && "z-10",
				isArchived && "hover:bg-transparent",
			)}
		>
			<div
				{...listeners}
				{...attributes}
				className={cn(
					"cursor-grab active:cursor-grabbing text-muted-foreground/60 hover:text-primary -ml-2 p-1 rounded-md hover:bg-primary/5",
					(isArchived || isEditing) && "hidden",
				)}
			>
				<GripVertical className="h-4 w-4" />
			</div>

			<button
				type="button"
				onClick={() => !isArchived && onToggle(listId, task.id, task.completed)}
				disabled={isArchived}
				className={cn(
					"shrink-0",
					task.completed
						? "text-primary/40"
						: "text-muted-foreground/20 hover:text-primary",
					isArchived && "cursor-default",
				)}
			>
				{task.completed ? (
					<Check className="h-5 w-5" />
				) : (
					<Circle className="h-5 w-5" />
				)}
			</button>

			<div className="flex-1 min-w-0">
				{isEditing && !isArchived ? (
					<form onSubmit={handleRename}>
						<input
							ref={inputRef}
							className="text-[17px] leading-relaxed block w-full bg-transparent border-none outline-none border-b border-primary/20"
							value={editedTitle}
							onChange={(e) => setEditedTitle(e.target.value)}
							onBlur={handleRename}
							autoFocus
						/>
					</form>
				) : (
					<button
						type="button"
						className={cn(
							"text-[17px] leading-relaxed block truncate w-full text-left",
							task.completed
								? "text-muted-foreground/40 line-through decoration-1"
								: "text-foreground/90",
							!isArchived && "cursor-text",
						)}
						onClick={() => !isArchived && setIsEditing(true)}
					>
						{task.title}
					</button>
				)}
			</div>

			{!isArchived && !isEditing && (
				<div className="flex items-center gap-1 opacity-0 group-hover:opacity-100">
					{([0.5, 1, 2, 3, 5, 8] as const).map((p) => (
						<button
							key={p}
							type="button"
							onClick={() =>
								onUpdatePoints(listId, task.id, task.points === p ? undefined : p)
							}
							className={cn(
								"w-6 h-6 rounded-full text-[10px] font-black transition-all flex items-center justify-center",
								task.points === p
									? "bg-primary text-primary-foreground shadow-sm"
									: "text-muted-foreground/30 hover:bg-primary/10 hover:text-primary",
							)}
						>
							{p}
						</button>
					))}
					<div className="w-px h-4 bg-border/40 mx-1" />
					<button
						type="button"
						onClick={() => onDelete(listId, task.id)}
						className="p-1.5 text-muted-foreground/30 hover:text-destructive transition-colors"
					>
						<X className="h-4 w-4" />
					</button>
				</div>
			)}

			{task.points && (
				<div
					className={cn(
						"text-[10px] font-black flex items-center justify-center w-6 h-6 rounded-full ring-1 transition-all",
						!isArchived && "group-hover:hidden",
						task.completed
							? "text-muted-foreground/20 bg-muted/5 ring-muted-foreground/10"
							: "text-primary/60 bg-primary/5 ring-primary/10",
					)}
				>
					{task.points}
				</div>
			)}
		</div>
	);
}

// --- Main Tasks Page ---

export default function Tasks() {
	const { data: lists = [], isLoading } = useLists();
	const [searchParams] = useSearchParams();
	const activeListId = searchParams.get("list") || "default";

	const addTaskMutation = useAddTaskMutation();
	const toggleTaskMutation = useToggleTaskMutation();
	const deleteTaskMutation = useDeleteTaskMutation();
	const renameTaskMutation = useRenameTaskMutation();
	const updatePointsMutation = useUpdateTaskPointsMutation();
	const renameListMutation = useRenameListMutation();
	const archiveListMutation = useArchiveListMutation();
	const updateJournalMutation = useUpdateJournalMutation();

	const [newTaskTitle, setNewTaskTitle] = useState("");
	const [isEditingTitle, setIsEditingTitle] = useState(false);
	const [editedTitle, setEditedTitle] = useState("");
	const [showJournal, setShowJournal] = useState(false);
	const [journalText, setJournalText] = useState("");
	const inputRef = useRef<HTMLInputElement>(null);
	const titleInputRef = useRef<HTMLInputElement>(null);
	const journalRef = useRef<HTMLTextAreaElement>(null);

	const activeList =
		lists.find((l) => l.id === activeListId) ||
		lists.find((l) => !l.archived) ||
		lists[0];

	useEffect(() => {
		if (activeList) {
			setJournalText(activeList.journal || "");
		}
	}, [activeList?.id, activeList?.journal]);

	const navigate = useNavigate();

	const handleJournalBlur = () => {
		if (activeList && journalText !== (activeList.journal || "")) {
			updateJournalMutation.mutate({ id: activeList.id, journal: journalText });
		}
	};

	const handleArchiveList = () => {
		if (activeList) {
			const isArchived = !!activeList.archived;
			archiveListMutation.mutate({ id: activeList.id, archived: !isArchived });

			// If we are archiving the list we are currently looking at,
			// navigate to the first available non-archived list.
			if (!isArchived) {
				const nextList = lists.find(
					(l) => l.id !== activeList.id && !l.archived,
				);
				if (nextList) {
					navigate(`/?list=${nextList.id}`);
				} else {
					navigate("/");
				}
			}
		}
	};

	const handleAddTask = (e: React.FormEvent) => {
		e.preventDefault();
		if (newTaskTitle.trim() && activeList) {
			addTaskMutation.mutate(
				{
					listId: activeList.id,
					title: newTaskTitle.trim(),
				},
				{
					onSettled: () => {
						inputRef.current?.focus();
					},
				},
			);
			setNewTaskTitle("");
		}
	};

	const handleRenameList = (e?: React.FormEvent) => {
		e?.preventDefault();
		if (editedTitle.trim() && activeList && editedTitle !== activeList.name) {
			renameListMutation.mutate({
				id: activeList.id,
				name: editedTitle.trim(),
			});
		}
		setIsEditingTitle(false);
	};

	const startEditingTitle = () => {
		if (activeList) {
			setEditedTitle(activeList.name);
			setIsEditingTitle(true);
			setTimeout(() => titleInputRef.current?.focus(), 0);
		}
	};

	const handleToggleTask = (listId: string, taskId: string, currentCompleted: boolean) => {
		toggleTaskMutation.mutate({ listId, taskId, currentCompleted });
	};

	const handleDeleteTask = (listId: string, taskId: string) => {
		deleteTaskMutation.mutate({ listId, taskId });
	};

	const handleRenameTask = (listId: string, taskId: string, title: string) => {
		renameTaskMutation.mutate({ listId, taskId, title });
	};

	const handleUpdatePoints = (
		listId: string,
		taskId: string,
		points?: 0.5 | 1 | 2 | 3 | 5 | 8,
	) => {
		updatePointsMutation.mutate({ listId, taskId, points });
	};

	if (isLoading) return null;

	return (
		<div
			className={cn(
				"flex-1 flex flex-col max-w-2xl mx-auto w-full px-6 pt-20 sm:pt-24 pb-12",
			)}
		>
			{activeList ? (
				<div className="flex flex-col gap-8">
					<header className="flex flex-col">
						<div className="flex items-center justify-between gap-4 mb-3">
							{isEditingTitle && !activeList.archived ? (
								<form onSubmit={handleRenameList} className="flex-1">
									<input
										ref={titleInputRef}
										className="text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight bg-transparent border-none outline-none w-full border-b border-primary/20"
										value={editedTitle}
										onChange={(e) => setEditedTitle(e.target.value)}
										onBlur={handleRenameList}
									/>
								</form>
							) : (
								<button
									type="button"
									className={cn(
										"text-3xl sm:text-4xl md:text-5xl font-bold tracking-tight text-left truncate flex-1 transition-colors",
										activeList.archived
											? "cursor-default text-foreground/60"
											: "cursor-pointer hover:text-primary/80",
									)}
									onClick={() => !activeList.archived && startEditingTitle()}
								>
									{activeList.name}
								</button>
							)}
							<div className="flex items-center gap-1">
								{!activeList.archived && (
									<button
										type="button"
										onClick={() => setShowJournal(!showJournal)}
										className={cn(
											"p-2 transition-colors shrink-0 rounded-full hover:bg-accent/10",
											showJournal || activeList.journal
												? "text-primary"
												: "text-muted-foreground/30 hover:text-primary",
										)}
										title="Journal for this list"
									>
										<FileText className="h-6 w-6" />
									</button>
								)}
								<button
									type="button"
									onClick={handleArchiveList}
									className={cn(
										"p-2 transition-colors shrink-0 rounded-full hover:bg-accent/10",
										activeList.archived
											? "text-primary hover:text-primary/80"
											: "text-muted-foreground/30 hover:text-primary",
									)}
									title={activeList.archived ? "Unarchive list" : "Archive list"}
								>
									<Archive className="h-6 w-6" />
								</button>
							</div>
						</div>
						<p className="text-sm font-medium text-muted-foreground/60 tracking-wide uppercase">
							{activeList.tasks.reduce((sum, t) => sum + (t.completed ? (t.points || 0) : 0), 0)} of{" "}
							{activeList.tasks.reduce((sum, t) => sum + (t.points || 0), 0)} pts
							{activeList.archived && " · ARCHIVED"}
						</p>
					</header>

					{activeList.archived && activeList.journal && (
						<div className="bg-accent/5 rounded-2xl p-6 border border-border/10 shadow-sm">
							<p className="text-foreground/70 text-[15px] leading-relaxed whitespace-pre-wrap font-medium italic">
								{activeList.journal}
							</p>
						</div>
					)}

					{!activeList.archived && showJournal && (
						<div className="animate-in fade-in slide-in-from-top-4 duration-300">
							<textarea
								ref={journalRef}
								className="w-full min-h-[160px] bg-accent/5 rounded-2xl p-6 text-foreground/80 placeholder:text-muted-foreground/20 border-none outline-none resize-none text-[15px] leading-relaxed ring-1 ring-border/20 focus:ring-primary/20 transition-shadow shadow-sm"
								placeholder="What's on your mind regarding this list? Record wins, challenges, or notes..."
								value={journalText}
								onChange={(e) => setJournalText(e.target.value)}
								onBlur={handleJournalBlur}
								autoFocus
							/>
						</div>
					)}

					<div className="flex flex-col gap-4">
						{!activeList.archived && (
							<form onSubmit={handleAddTask} className="relative group">
								<input
									ref={inputRef}
									className="w-full text-xl bg-transparent border-none outline-none placeholder:text-muted-foreground/20 border-b border-transparent group-focus-within:border-primary/20 py-3"
									placeholder="Capture your next step..."
									value={newTaskTitle}
									onChange={(e) => setNewTaskTitle(e.target.value)}
								/>
							</form>
						)}
						<div className="flex flex-col">
							{(() => {
								const displayTasks = [
									...activeList.tasks.filter((t) => !t.completed),
									...activeList.tasks.filter((t) => t.completed),
								];
								return (
									<SortableContext
										items={displayTasks.map((t) => t.id)}
										strategy={verticalListSortingStrategy}
									>
										{displayTasks.map((task) => (
											<DraggableTask
												key={task.id}
												task={task}
												listId={activeList.id}
												onToggle={handleToggleTask}
												onDelete={handleDeleteTask}
												onUpdatePoints={handleUpdatePoints}
												onRename={handleRenameTask}
												isArchived={activeList.archived}
											/>
										))}
									</SortableContext>
								);
							})()}

							{activeList.tasks.length === 0 && (
								<div className="py-20 text-center">
									<p className="text-muted-foreground/30 text-sm italic font-medium">
										Your canvas is empty.
									</p>
								</div>
							)}
						</div>
					</div>
				</div>
			) : (
				<div className="flex flex-col items-center justify-center h-full text-muted-foreground/40">
					<ListIcon className="h-12 w-12 mb-4 opacity-10" />
					<p className="text-sm font-medium tracking-tight">
						Select a collection to begin.
					</p>
				</div>
			)}
		</div>
	);
}
