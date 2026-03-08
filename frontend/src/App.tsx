import {
	createBrowserRouter,
	Navigate,
	RouterProvider,
} from "react-router-dom";
import Layout from "./components/Layout";
import ReloadPrompt from "./components/ReloadPrompt";
import Archive from "./pages/Archive";
import Dashboard from "./pages/Dashboard";
import Login from "./pages/Login";
import Tasks from "./pages/Tasks";

export const router = createBrowserRouter([
	{
		path: "/login",
		element: <Login />,
	},
	{
		path: "/",
		element: <Layout />,
		children: [
			{
				index: true,
				element: <Tasks />,
			},
			{
				path: "dashboard",
				element: <Dashboard />,
			},
			{
				path: "archive",
				element: <Archive />,
			},
		],
	},
	{
		path: "*",
		element: <Navigate to="/" replace />,
	},
]);

export default function App() {
	return (
		<>
			<RouterProvider router={router} />
			<ReloadPrompt />
		</>
	);
}
