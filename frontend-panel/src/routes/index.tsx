import { createBrowserRouter } from "react-router-dom";
import { AdminShell } from "@/features/admin/AdminShell";
import { AuthGate } from "@/features/admin/AuthGate";
import { AdminActivityPage } from "@/pages/AdminActivityPage";
import { AdminClassificationPage } from "@/pages/AdminClassificationPage";
import { AdminDevicesPage } from "@/pages/AdminDevicesPage";
import { AdminPage } from "@/pages/AdminPage";
import { AdminSessionsPage } from "@/pages/AdminSessionsPage";
import { AdminSettingsPage } from "@/pages/AdminSettingsPage";
import { ErrorPage } from "@/pages/ErrorPage";
import { PublicStatusPage } from "@/pages/PublicStatusPage";
import { appPaths } from "@/routes/routePaths";

export const router = createBrowserRouter([
	{
		path: appPaths.publicStatus,
		element: <PublicStatusPage />,
		errorElement: <ErrorPage />,
	},
	{
		path: appPaths.admin,
		element: (
			<AuthGate>
				<AdminShell />
			</AuthGate>
		),
		errorElement: <ErrorPage />,
		children: [
			{
				index: true,
				element: <AdminPage />,
			},
			{
				path: "devices",
				element: <AdminDevicesPage />,
			},
			{
				path: "activity",
				element: <AdminActivityPage />,
			},
			{
				path: "settings",
				element: <AdminSettingsPage />,
			},
			{
				path: "classification",
				element: <AdminClassificationPage />,
			},
			{
				path: "sessions",
				element: <AdminSessionsPage />,
			},
		],
	},
	{
		path: "*",
		element: <ErrorPage />,
	},
]);
