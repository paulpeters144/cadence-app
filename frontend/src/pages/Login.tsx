import { Navigate, useNavigate } from "react-router-dom";
import { Button } from "../components/ui/button";
import { useLoginMutation, useUser } from "../lib/queries";

export default function Login() {
	const navigate = useNavigate();
	const { data: user, isLoading: isUserLoading } = useUser();
	const loginMutation = useLoginMutation();

	if (isUserLoading) return null;
	if (user) return <Navigate to="/" replace />;

	const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();
		const formData = new FormData(e.currentTarget);
		const username = formData.get("username") as string;
		const password = formData.get("password") as string;
		loginMutation.mutate(
			{ username, password },
			{
				onSuccess: () => navigate("/", { replace: true }),
			},
		);
	};

	return (
		<div className="flex min-h-screen items-center justify-center bg-background p-4">
			<div className="w-full max-w-sm space-y-8 rounded-2xl border border-border/40 bg-card p-8 shadow-xl">
				<div className="text-center space-y-2">
					<h1 className="text-3xl font-bold tracking-tight text-foreground">
						Welcome Back
					</h1>
					<p className="text-sm text-muted-foreground">
						Enter your credentials to access Cadence
					</p>
				</div>
				<form onSubmit={handleSubmit} className="space-y-6">
					<div className="space-y-2">
						<label
							htmlFor="username"
							className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
						>
							Username
						</label>
						<input
							id="username"
							name="username"
							type="text"
							placeholder="username"
							required
							disabled={loginMutation.isPending}
							className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						/>
					</div>
					<div className="space-y-2">
						<label
							htmlFor="password"
							className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
						>
							Password
						</label>
						<input
							id="password"
							name="password"
							type="password"
							placeholder="••••••••"
							required
							disabled={loginMutation.isPending}
							className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
						/>
					</div>
					<Button
						type="submit"
						className="w-full"
						disabled={loginMutation.isPending}
					>
						{loginMutation.isPending ? "Signing In..." : "Sign In"}
					</Button>
				</form>
				<p className="text-center text-xs text-muted-foreground">
					This is an app to measure your day to day cadence.
				</p>
			</div>
		</div>
	);
}
