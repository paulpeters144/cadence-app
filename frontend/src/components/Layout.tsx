import {
	DndContext,
	type DragEndEvent,
	DragOverlay,
	type DragStartEvent,
	MouseSensor,
	TouchSensor,
	useDroppable,
	useSensor,
	useSensors,
} from "@dnd-kit/core";
import {
	SortableContext,
	useSortable,
	verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
	Archive,
	BarChart2,
	Copy,
	GripVertical,
	List as ListIcon,
	LogOut,
	Menu,
	MoreHorizontal,
	Pencil,
	Plus,
	Trash2,
	X,
} from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Link, Navigate, Outlet, useLocation } from "react-router-dom";
import {
	useAddListMutation,
	useDeleteListMutation,
	useDuplicateListMutation,
	useLists,
	useLogoutMutation,
	useMoveTaskMutation,
	useRenameListMutation,
	useReorderListsMutation,
	useReorderTasksMutation,
	useUser,
} from "../lib/queries";
import type { List, Task } from "../lib/types";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuTrigger,
} from "./ui/dropdown-menu";

function SortableListItem({
	list,
	activeListId,
	onDelete,
	onRename,
	onDuplicate,
}: {
	list: List;
	activeListId: string | null;
	onDelete: (id: string) => void;
	onRename: (id: string, name: string) => void;
	onDuplicate: (id: string, name: string) => void;
}) {
	const {
		attributes,
		listeners,
		setNodeRef,
		transform,
		transition,
		isDragging,
		isOver,
	} = useSortable({
		id: list.id,
		data: {
			type: "list",
			list,
		},
	});

	const [isEditing, setIsEditing] = useState(false);
	const [newName, setNewName] = useState(list.name);
	const inputRef = useRef<HTMLInputElement>(null);

	const style = {
		transform: CSS.Translate.toString(transform),
		transition,
		opacity: isDragging ? 0.4 : 1,
	};

	const handleRename = (e?: React.FormEvent) => {
		e?.preventDefault();
		if (newName.trim() && newName !== list.name) {
			onRename(list.id, newName.trim());
		}
		setIsEditing(false);
	};

	if (isEditing) {
		return (
			<div
				ref={setNodeRef}
				style={style}
				className="flex items-center px-4 py-1.5"
			>
				<form onSubmit={handleRename} className="flex-1">
					<input
						ref={inputRef}
						className="w-full bg-transparent border-b border-primary/20 py-0.5 outline-none text-sm"
						value={newName}
						onChange={(e) => setNewName(e.target.value)}
						onBlur={handleRename}
					/>
				</form>
			</div>
		);
	}

	return (
		<div
			ref={setNodeRef}
			style={style}
			className={cn(
				"group flex items-center px-1 py-0.5 rounded-lg",
				isOver && !isDragging && "bg-primary/5 ring-1 ring-primary/20",
			)}
		>
			<Link
				to={`/?list=${list.id}`}
				className={cn(
					"flex-1 flex items-center justify-between px-3 py-1.5 rounded-md cursor-pointer overflow-hidden",
					activeListId === list.id && window.location.pathname === "/"
						? "bg-primary/5 text-primary font-bold shadow-[inset_0_0_0_1px_rgba(0,0,0,0.05)]"
						: "hover:bg-accent/50 text-muted-foreground hover:text-foreground",
				)}
			>
				<div className="flex items-center gap-2.5 overflow-hidden">
					<div
						{...attributes}
						{...listeners}
						className="cursor-grab active:cursor-grabbing"
					>
						<ListIcon
							className={cn(
								"h-3.5 w-3.5 shrink-0",
								activeListId === list.id ? "opacity-100" : "opacity-40",
							)}
						/>
					</div>
					<span className="truncate text-sm font-medium leading-none">
						{list.name}
					</span>
				</div>
				<div className="flex items-center opacity-0 group-hover:opacity-100">
					<DropdownMenu>
						<DropdownMenuTrigger asChild>
							<button
								type="button"
								onClick={(e) => {
									e.stopPropagation();
									e.preventDefault();
								}}
								className="p-1 hover:text-foreground text-muted-foreground/60 transition-colors outline-none"
							>
								<MoreHorizontal className="h-3.5 w-3.5" />
							</button>
						</DropdownMenuTrigger>
						<DropdownMenuContent align="end" className="w-32">
							<DropdownMenuItem
								onClick={(e) => {
									e.stopPropagation();
									setIsEditing(true);
									setTimeout(() => inputRef.current?.focus(), 0);
								}}
							>
								<Pencil className="mr-2 h-3.5 w-3.5" />
								<span>Rename</span>
							</DropdownMenuItem>
							<DropdownMenuItem
								onClick={(e) => {
									e.stopPropagation();
									onDuplicate(list.id, list.name);
								}}
							>
								<Copy className="mr-2 h-3.5 w-3.5" />
								<span>Duplicate</span>
							</DropdownMenuItem>
							<DropdownMenuItem
								className="text-destructive focus:text-destructive"
								onClick={(e) => {
									e.stopPropagation();
									if (confirm("Delete this list?")) onDelete(list.id);
								}}
							>
								<Trash2 className="mr-2 h-3.5 w-3.5" />
								<span>Delete</span>
							</DropdownMenuItem>
						</DropdownMenuContent>
					</DropdownMenu>
				</div>
			</Link>
		</div>
	);
}

function SidebarToggleButton({
	isOpen,
	onClick,
}: {
	isOpen: boolean;
	onClick: () => void;
}) {
	const { setNodeRef, isOver } = useDroppable({
		id: "sidebar-toggle",
	});

	useEffect(() => {
		if (isOver && !isOpen && window.innerWidth < 1024) {
			onClick();
		}
	}, [isOver, isOpen, onClick]);

	return (
		<Button
			ref={setNodeRef}
			variant="ghost"
			size="icon"
			className={cn(
				"h-8 w-8 text-muted-foreground/40 hover:text-foreground",
				isOver &&
					"text-primary bg-primary/10 shadow-lg ring-2 ring-primary/20 ring-offset-2 ring-offset-background",
			)}
			onClick={onClick}
		>
			<Menu className="h-4 w-4" />
		</Button>
	);
}

export default function Layout() {
	const { data: user, isLoading: isUserLoading } = useUser();
	const { data: lists = [] } = useLists();

	const logoutMutation = useLogoutMutation();
	const addListMutation = useAddListMutation();
	const deleteListMutation = useDeleteListMutation();
	const duplicateListMutation = useDuplicateListMutation();
	const renameListMutation = useRenameListMutation();
	const reorderListsMutation = useReorderListsMutation();
	const reorderTasksMutation = useReorderTasksMutation();
	const moveTaskMutation = useMoveTaskMutation();

	const location = useLocation();
	const searchParams = new URLSearchParams(location.search);
	const activeListId = searchParams.get("list") || "default";

	const [newListName, setNewListName] = useState("");
	const [showAddList, setShowAddList] = useState(false);
	const [isSidebarOpen, setIsSidebarOpen] = useState(
		() => window.innerWidth >= 1024,
	);
	const [activeDragTask, setActiveDragTask] = useState<Task | null>(null);
	const [activeDragList, setActiveDragList] = useState<List | null>(null);

	const [isDarkMode, setIsDarkMode] = useState(() => {
		if (typeof window !== "undefined") {
			const saved = localStorage.getItem("theme");
			return saved === "dark" || (!saved && window.matchMedia("(prefers-color-scheme: dark)").matches);
		}
		return false;
	});

	useEffect(() => {
		if (isDarkMode) {
			document.documentElement.classList.add("dark");
			localStorage.setItem("theme", "dark");
		} else {
			document.documentElement.classList.remove("dark");
			localStorage.setItem("theme", "light");
		}
	}, [isDarkMode]);

	const sensors = useSensors(
		useSensor(MouseSensor, {
			activationConstraint: { distance: 5 },
		}),
		useSensor(TouchSensor, {
			activationConstraint: { delay: 250, tolerance: 5 },
		}),
	);

	useEffect(() => {
		const down = (e: KeyboardEvent) => {
			if (e.key === "b" && (e.metaKey || e.ctrlKey)) {
				e.preventDefault();
				setIsSidebarOpen((prev) => !prev);
			}
		};

		document.addEventListener("keydown", down);
		return () => document.removeEventListener("keydown", down);
	}, []);

	useEffect(() => {
		if (window.innerWidth < 1024) {
			setIsSidebarOpen(false);
		}
	}, [location]);

	const newListInputRef = useRef<HTMLInputElement>(null);

	if (isUserLoading) {
		return null;
	}

	if (!user) {
		return <Navigate to="/login" replace />;
	}

	const handleAddList = (e: React.FormEvent) => {
		e.preventDefault();
		if (newListName.trim()) {
			addListMutation.mutate(newListName.trim());
			setNewListName("");
			setShowAddList(false);
		}
	};

	const handleDeleteList = (id: string) => {
		deleteListMutation.mutate(id);
	};

	const handleDuplicateList = (id: string, name: string) => {
		// Logic to find the next available incrementing number
		const baseName = name.replace(/\s\d+$/, "");
		const existingNumbers = lists
			.filter((l) => l.name.startsWith(baseName))
			.map((l) => {
				const match = l.name.match(/\s(\d+)$/);
				return match ? parseInt(match[1], 10) : 0;
			});
		const nextNumber = existingNumbers.length > 0 ? Math.max(...existingNumbers) + 1 : 1;
		const newName = `${baseName} ${nextNumber}`;

		duplicateListMutation.mutate({ id, newName });
	};

	const handleRenameList = (id: string, name: string) => {
		renameListMutation.mutate({ id, name });
	};

	const handleDragStart = (event: DragStartEvent) => {
		const { active } = event;
		const { type, task, list } = active.data.current || {};

		if (type === "list") {
			setActiveDragList(list);
		} else if (task) {
			setActiveDragTask(task);
		}
	};

	const handleDragEnd = (event: DragEndEvent) => {
		const { active, over } = event;
		setActiveDragTask(null);
		setActiveDragList(null);

		if (!over) return;

		const activeData = active.data.current;
		const overData = over.data.current;

		const activeType = activeData?.type;
		const overType = overData?.type;

		if (activeType === "list") {
			if (active.id !== over.id && overType === "list") {
				reorderListsMutation.mutate({
					activeId: active.id as string,
					overId: over.id as string,
				});
			}
		} else {
			// Task dragging
			const taskId = active.id as string;
			const fromListId = activeData?.listId;

			if (overType === "list") {
				// Dropped on a list in sidebar
				const toListId = over.id as string;
				if (fromListId !== toListId) {
					moveTaskMutation.mutate({
						fromListId,
						toListId,
						taskId,
					});
				}
			} else {
				// Dropped on another task
				const toListId = overData?.listId;
				if (fromListId === toListId) {
					// Reorder within same list
					if (active.id !== over.id) {
						reorderTasksMutation.mutate({
							listId: fromListId,
							activeId: active.id as string,
							overId: over.id as string,
						});
					}
				} else {
					// Moving between lists by dropping on a task in another list
					moveTaskMutation.mutate({
						fromListId,
						toListId,
						taskId,
					});
				}
			}
		}
	};

	const handleLogout = () => {
		logoutMutation.mutate();
	};

	return (
		<DndContext
			sensors={sensors}
			onDragStart={handleDragStart}
			onDragEnd={handleDragEnd}
		>
			<div className="flex min-h-screen bg-background text-foreground font-sans antialiased selection:bg-primary/10">
				{/* Sidebar */}
				{isSidebarOpen && (
					<>
						<button
							type="button"
							onClick={() => setIsSidebarOpen(false)}
							aria-label="Close sidebar"
							className="fixed inset-0 bg-background/40 z-40 lg:hidden cursor-default"
						/>
						<aside className="fixed inset-y-0 left-0 z-50 lg:relative lg:inset-auto lg:z-0 border-r border-border/40 bg-sidebar overflow-hidden flex flex-col shadow-2xl lg:shadow-none w-[260px]">
							<div className="p-6 flex flex-col gap-8 h-full">
								<div className="flex items-center justify-between">
									<h2 className="text-xs font-bold uppercase tracking-widest text-muted-foreground/60">
										Collections
									</h2>
									<div className="flex items-center gap-2">
										<button
											type="button"
											onClick={() => setShowAddList(!showAddList)}
											className="text-muted-foreground hover:text-foreground p-1"
										>
											<Plus className="h-4 w-4" />
										</button>
										<button
											type="button"
											onClick={() => setIsSidebarOpen(false)}
											className="lg:hidden text-muted-foreground hover:text-foreground p-1"
										>
											<X className="h-4 w-4" />
										</button>
									</div>
								</div>

								<div className="flex flex-col gap-1 overflow-y-auto pr-2 -mr-2">
									<Link
										to="/dashboard"
										className={cn(
											"flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm",
											location.pathname === "/dashboard"
												? "font-bold bg-primary/5 text-primary shadow-[inset_0_0_0_1px_rgba(0,0,0,0.05)]"
												: "font-medium text-muted-foreground hover:bg-accent/50 hover:text-foreground",
										)}
									>
										<BarChart2
											className={cn(
												"h-3.5 w-3.5",
												location.pathname === "/dashboard"
													? "opacity-100"
													: "opacity-40",
											)}
										/>
										Dashboard
									</Link>

									<Link
										to="/archive"
										className={cn(
											"flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm mb-4",
											location.pathname === "/archive"
												? "font-bold bg-primary/5 text-primary shadow-[inset_0_0_0_1px_rgba(0,0,0,0.05)]"
												: "font-medium text-muted-foreground hover:bg-accent/50 hover:text-foreground",
										)}
									>
										<Archive
											className={cn(
												"h-3.5 w-3.5",
												location.pathname === "/archive"
													? "opacity-100"
													: "opacity-40",
											)}
										/>
										Archive
									</Link>

									{showAddList && (
										<form onSubmit={handleAddList} className="mb-4">
											<input
												ref={newListInputRef}
												className="w-full bg-transparent border-b border-primary/20 py-1 outline-none text-sm placeholder:text-muted-foreground/40"
												placeholder="Name your list..."
												value={newListName}
												onChange={(e) => setNewListName(e.target.value)}
												onBlur={() => !newListName && setShowAddList(false)}
											/>
										</form>
									)}

									<div className="flex flex-col gap-0.5">
										<SortableContext
											items={lists.filter((l) => !l.archived).map((l) => l.id)}
											strategy={verticalListSortingStrategy}
										>
											{lists
												.filter((l) => !l.archived)
												.map((list) => (
													<SortableListItem
														key={list.id}
														list={list}
														activeListId={activeListId}
														onDelete={handleDeleteList}
														onRename={handleRenameList}
														onDuplicate={handleDuplicateList}
													/>
												))}
										</SortableContext>
									</div>
								</div>

								<div className="mt-auto pt-6 border-t border-border/40 flex flex-col gap-4">
									<div className="flex items-center justify-between px-2">
										<span className="text-[10px] font-medium text-muted-foreground/60 uppercase tracking-wider">
											Dark Mode
										</span>
										<button
											type="button"
											onClick={() => setIsDarkMode(!isDarkMode)}
											className={`relative inline-flex h-4 w-8 shrink-0 cursor-pointer items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ${
												isDarkMode ? "bg-foreground" : "bg-muted"
											}`}
										>
											<span
												className={`pointer-events-none block h-3 w-3 rounded-full bg-background shadow-sm transition-transform ${
													isDarkMode ? "translate-x-4" : "translate-x-1"
												}`}
											/>
										</button>
									</div>

									<div className="flex items-center justify-between px-2">
										<div className="flex flex-col min-w-0">
											<p className="text-[10px] text-muted-foreground truncate">
												Logged in as
											</p>
											<p className="text-xs font-medium truncate">
												{user.username}
											</p>
										</div>
										<button
											type="button"
											onClick={handleLogout}
											className="p-1.5 text-muted-foreground hover:text-destructive rounded-md hover:bg-destructive/5"
											title="Logout"
										>
											<LogOut className="h-4 w-4" />
										</button>
									</div>
								</div>
							</div>
						</aside>
					</>
				)}

				{/* Main Content */}
				<main className="flex-1 flex flex-col relative overflow-y-auto">
					{/* Floating Controls */}
					<div className="absolute top-6 left-6 flex gap-2 z-10">
						<SidebarToggleButton
							isOpen={isSidebarOpen}
							onClick={() => setIsSidebarOpen(!isSidebarOpen)}
						/>
					</div>

					<Outlet />
				</main>
			</div>

			<DragOverlay dropAnimation={null}>
				{activeDragTask ? (
					<div className="flex items-center gap-4 py-4 px-4 bg-background border border-primary/20 shadow-2xl rounded-xl opacity-90 cursor-grabbing w-[400px]">
						<GripVertical className="h-4 w-4 text-primary" />
						<div className="flex-1 min-w-0">
							<span className="text-[17px] leading-relaxed block truncate">
								{activeDragTask.title}
							</span>
						</div>
					</div>
				) : activeDragList ? (
					<div className="flex items-center gap-2.5 px-4 py-3 rounded-lg bg-background border border-primary/20 shadow-xl opacity-90 cursor-grabbing w-[240px]">
						<ListIcon className="h-3.5 w-3.5 text-primary" />
						<span className="text-sm font-medium">{activeDragList.name}</span>
					</div>
				) : null}
			</DragOverlay>
		</DndContext>
	);
}
