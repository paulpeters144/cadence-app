import {
	BarChart2,
	ChevronLeft,
	ChevronRight,
} from "lucide-react";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useLists } from "../lib/queries";
import { cn } from "../lib/utils";

export default function Dashboard() {
	const { data: lists = [], isLoading } = useLists();
	const navigate = useNavigate();

	const [timeRange, setTimeRange] = useState<7 | 30>(7);
	const [dateOffset, setDateOffset] = useState(0);

	if (isLoading) return null;

	// Calculate stats
	const archivedLists = lists.filter((l) => l.archived);
	const allTasks = archivedLists.flatMap((l) => l.tasks);
	const completedTasks = allTasks.filter((t) => t.completed && t.completedAt);

	const endDate = new Date();
	endDate.setHours(23, 59, 59, 999);
	endDate.setDate(endDate.getDate() - dateOffset * timeRange);

	const days = Array.from({ length: timeRange }, (_, i) => {
		const d = new Date(endDate);
		d.setHours(0, 0, 0, 0);
		d.setDate(d.getDate() - (timeRange - 1 - i));
		return d;
	});

	const startDate = days[0];

	const statsPerDay = days.map((day) => {
		const dayEnd = new Date(day);
		dayEnd.setDate(day.getDate() + 1);
		
		const dayTasks = completedTasks.filter((t) => {
			const completedDate = new Date(t.completedAt!);
			return completedDate >= day && completedDate < dayEnd;
		});

		const listsArchivedThisDay = archivedLists.filter((l) => {
			if (!l.archivedAt) return false;
			const archivedDate = new Date(l.archivedAt);
			return archivedDate >= day && archivedDate < dayEnd;
		});

		return {
			date: day,
			label:
				timeRange === 7
					? day.toLocaleDateString("en-US", { weekday: "short" })
					: day.getDate().toString(),
			points: dayTasks.reduce((sum, t) => sum + (t.points || 0), 0),
			archivedListIds: listsArchivedThisDay.map(l => l.id),
		};
	});

	const maxValue = Math.max(...statsPerDay.map((s) => s.points), 5);

	const handleBarClick = (day: typeof statsPerDay[0]) => {
		if (day.archivedListIds.length > 0) {
			const dateStr = day.date.toISOString().split("T")[0];
			navigate(`/archive?date=${dateStr}`);
		}
	};

	return (
		<div className="max-w-3xl mx-auto w-full px-6 pt-20 pb-12">
			<div className="flex items-center justify-between mb-12">
				<div>
					<h1 className="text-3xl font-bold tracking-tight">Dashboard</h1>
					<p className="text-sm text-muted-foreground">
						POC - Stats Visualization
					</p>
				</div>
			</div>

			<div className="flex items-center justify-between mb-6">
				<div className="flex bg-accent/30 p-1 rounded-xl border border-border/40">
					{([7, 30] as const).map((range) => (
						<button
							key={range}
							type="button"
							onClick={() => {
								setTimeRange(range);
								setDateOffset(0);
							}}
							className={cn(
								"px-4 py-1.5 text-xs font-bold uppercase tracking-wider rounded-lg",
								timeRange === range
									? "bg-background shadow-sm text-primary"
									: "text-muted-foreground/60 hover:text-foreground",
							)}
						>
							{range}D
						</button>
					))}
				</div>

				<div className="flex items-center gap-2">
					<button
						type="button"
						onClick={() => setDateOffset((prev) => prev + 1)}
						className="p-1.5 rounded-lg hover:bg-accent text-muted-foreground border border-border/40"
					>
						<ChevronLeft className="h-4 w-4" />
					</button>
					<span className="text-[10px] font-bold text-muted-foreground/60 uppercase tracking-widest min-w-[120px] text-center">
						{startDate.toLocaleDateString(undefined, {
							month: "short",
							day: "numeric",
						})}{" "}
						-{" "}
						{endDate.toLocaleDateString(undefined, {
							month: "short",
							day: "numeric",
						})}
					</span>
					<button
						type="button"
						onClick={() => setDateOffset((prev) => Math.max(0, prev - 1))}
						disabled={dateOffset === 0}
						className="p-1.5 rounded-lg hover:bg-accent text-muted-foreground border border-border/40 disabled:opacity-20"
					>
						<ChevronRight className="h-4 w-4" />
					</button>
				</div>
			</div>

			<div className="p-8 rounded-3xl bg-card border border-border/40 shadow-sm">
				<div className="flex items-center justify-between mb-10">
					<div className="flex items-center gap-2">
						<BarChart2 className="h-4 w-4 text-primary" />
						<h3 className="font-semibold">Performance History</h3>
					</div>
				</div>

				<div
					className={cn(
						"h-48 w-full flex items-end justify-between",
						timeRange === 7 ? "gap-4" : "gap-1",
					)}
				>
					{statsPerDay.map((day, i) => {
						const height = (day.points / maxValue) * 100;
						return (
							<div
								key={day.date.getTime()}
								onClick={() => handleBarClick(day)}
								className={cn(
									"flex-1 flex flex-col items-center gap-4 group min-w-0",
									day.archivedListIds.length > 0 && "cursor-pointer",
								)}
							>
								<div className="relative w-full flex flex-col items-center justify-end h-32">
									<div
										style={{ height: `${height}%` }}
										className={cn(
											"w-full rounded-t-lg transition-colors",
											day.points > 0 ? "bg-primary" : "bg-primary/5",
											day.archivedListIds.length > 0 && "group-hover:bg-primary/80",
											timeRange === 7 ? "max-w-[40px]" : "max-w-none",
										)}
									/>
									{day.points > 0 && (
										<div className="absolute -top-6 opacity-0 group-hover:opacity-100 z-10 pointer-events-none">
											<span className="text-[10px] font-bold bg-primary text-primary-foreground px-1.5 py-0.5 rounded whitespace-nowrap">
												{day.points} pts
											</span>
										</div>
									)}
								</div>
								<span
									className={cn(
										"text-[9px] sm:text-[10px] font-bold tracking-tight whitespace-nowrap",
										dateOffset === 0 && i === statsPerDay.length - 1
											? "text-primary"
											: "text-muted-foreground/40",
										timeRange === 30 && "rotate-45 sm:rotate-0",
									)}
								>
									{day.label}
								</span>
							</div>
						);
					})}
				</div>
			</div>
		</div>
	);
}
