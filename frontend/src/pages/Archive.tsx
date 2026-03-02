import { Archive as ArchiveIcon, Trash2 } from "lucide-react";
import { useSearchParams, Link } from "react-router-dom";
import { useLists, useArchiveListMutation, useDeleteListMutation } from "../lib/queries";

export default function Archive() {
	const { data: lists = [], isLoading } = useLists();
	const [searchParams] = useSearchParams();
	const archiveMutation = useArchiveListMutation();
	const deleteMutation = useDeleteListMutation();

	// Parse date from query param (e.g., ?date=2026-03-01)
	const dateParam = searchParams.get("date");

	if (isLoading) return null;

	const displayedLists = dateParam 
		? lists.filter(l => l.archived && l.archivedAt?.startsWith(dateParam))
		: lists.filter(l => l.archived);

	const handleDelete = (id: string) => {
		if (confirm("Permanently delete this list and all its tasks?")) {
			deleteMutation.mutate(id);
		}
	};

	const formatDate = (dateStr: string) => {
		return new Date(dateStr).toLocaleDateString(undefined, {
			month: "long",
			day: "numeric",
			year: "numeric"
		});
	};

	return (
		<div className="max-w-3xl mx-auto w-full px-6 pt-20 pb-12">
			<div className="flex items-center justify-between mb-12">
				<div>
					<h1 className="text-3xl font-bold tracking-tight">Archive</h1>
					<p className="text-sm text-muted-foreground">
						{dateParam ? `Archived on ${formatDate(dateParam)}` : "All Collections"}
					</p>
				</div>
			</div>

			<div className="grid gap-6">
				{displayedLists.length === 0 ? (
					<div className="py-20 text-center border-2 border-dashed border-border/40 rounded-3xl">
						<ArchiveIcon className="h-12 w-12 mx-auto mb-4 opacity-10" />
						<p className="text-muted-foreground/40 font-medium">
							No archived lists found.
						</p>
					</div>
				) : (
					displayedLists.map((list) => (
						<Link
							key={list.id}
							to={`/?list=${list.id}`}
							className="p-6 rounded-2xl bg-card border border-border/40 shadow-sm flex items-center justify-between group hover:border-primary/20 transition-all hover:shadow-md"
						>
							<div className="flex flex-col gap-1 min-w-0">
								<h3 className="font-bold text-lg truncate pr-4">{list.name}</h3>
								<p className="text-xs text-muted-foreground/60 uppercase tracking-widest font-black">
									{list.tasks.reduce((sum, t) => sum + (t.completed ? (t.points || 0) : 0), 0)} of{" "}
									{list.tasks.reduce((sum, t) => sum + (t.points || 0), 0)} pts
								</p>
							</div>
							<div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
								<button
									type="button"
									onClick={(e) => {
										e.preventDefault();
										e.stopPropagation();
										archiveMutation.mutate({ id: list.id, archived: !list.archived });
									}}
									className="px-4 py-2 rounded-xl bg-primary/10 hover:bg-primary/20 text-primary text-xs font-bold uppercase tracking-widest transition-colors"
								>
									{list.archived ? "Unarchive" : "Archive"}
								</button>
								<button
									type="button"
									onClick={(e) => {
										e.preventDefault();
										e.stopPropagation();
										handleDelete(list.id);
									}}
									className="p-2.5 rounded-xl bg-destructive/10 hover:bg-destructive/20 text-destructive transition-colors"
								>
									<Trash2 className="h-4 w-4" />
								</button>
							</div>
						</Link>
					))
				)}
			</div>
		</div>
	);
}
