import { spawn } from "node:child_process";
import * as os from "node:os";

console.log("🚀 Starting the Cadence App Backend in a new window...");

const platform = os.platform();
const backendCommand = "pnpm run dev:backend";

if (platform === "win32") {
	// Open the backend in a new separate PowerShell window on Windows
	spawn(
		"powershell.exe",
		["-NoProfile", "-Command", `Start-Process powershell -ArgumentList '${backendCommand}'`],
		{ stdio: "inherit" }
	);
} else if (platform === "darwin") {
	// Open a new Terminal window on macOS
	spawn(
		"osascript",
		[
			"-e",
			`tell app "Terminal" to do script "cd '${process.cwd()}' && ${backendCommand}"`
		],
		{ stdio: "inherit" }
	);
} else {
	// For Linux, try common terminal emulators
	const linuxScript = `
if command -v x-terminal-emulator >/dev/null 2>&1; then
    x-terminal-emulator -e "${backendCommand}"
elif command -v gnome-terminal >/dev/null 2>&1; then
    gnome-terminal -- bash -c "${backendCommand}; exec bash"
elif command -v konsole >/dev/null 2>&1; then
    konsole -e "${backendCommand}"
elif command -v xfce4-terminal >/dev/null 2>&1; then
    xfce4-terminal -e "${backendCommand}"
elif command -v alacritty >/dev/null 2>&1; then
    alacritty -e bash -c "${backendCommand}"
else
    echo "No supported terminal emulator found. Starting backend in the background instead."
    ${backendCommand} &
fi
`;
	spawn("sh", ["-c", linuxScript], { stdio: "inherit" });
}

console.log("🚀 Starting the Cadence App Frontend in the current terminal...");

// Start the frontend in the current terminal window
const frontendProcess = spawn("pnpm", ["run", "dev:frontend"], {
	stdio: "inherit",
	shell: true,
});

frontendProcess.on("close", (code) => {
	console.log(`Frontend process exited with code ${code}`);
	process.exit(code ?? 0);
});

frontendProcess.on("error", (err) => {
	console.error("Failed to start frontend process:", err);
	process.exit(1);
});
