import { Button } from "@/components/ui/button";
import { useRegisterSW } from "virtual:pwa-register/react";

function ReloadPrompt() {
	const {
		offlineReady: [offlineReady, setOfflineReady],
		needRefresh: [needRefresh, setNeedRefresh],
		updateServiceWorker,
	} = useRegisterSW({
		onRegistered(r) {
			console.log("SW Registered: ", r);
		},
		onRegisterError(error) {
			console.error("SW registration error", error);
		},
	});

	const close = () => {
		setOfflineReady(false);
		setNeedRefresh(false);
	};

	if (!offlineReady && !needRefresh) {
		return null;
	}

	return (
		<div className="fixed bottom-4 right-4 z-50 p-4 bg-background border border-border rounded-lg shadow-lg flex flex-col gap-3 min-w-[300px]">
			<div className="text-sm">
				{offlineReady ? (
					<span>App ready to work offline</span>
				) : (
					<span>New content available, click on reload button to update.</span>
				)}
			</div>
			<div className="flex gap-2 justify-end">
				{needRefresh && (
					<Button size="sm" onClick={() => updateServiceWorker(true)}>
						Reload
					</Button>
				)}
				<Button size="sm" variant="outline" onClick={() => close()}>
					Close
				</Button>
			</div>
		</div>
	);
}

export default ReloadPrompt;
