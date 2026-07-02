import { useTranslation } from "react-i18next";
import { Link, NavLink, Outlet } from "react-router-dom";
import { appPaths } from "@/routes/routePaths";
import { useAuthStore } from "@/stores/authStore";
import { AdminPanel } from "./AdminPanel";

const navItems = [
	{ to: appPaths.admin, labelKey: "nav.overview" },
	{ to: appPaths.adminDevices, labelKey: "nav.devices" },
	{ to: appPaths.adminActivity, labelKey: "nav.activity" },
	{ to: appPaths.adminSettings, labelKey: "nav.settings" },
	{ to: appPaths.adminClassification, labelKey: "nav.classification" },
	{ to: appPaths.adminSessions, labelKey: "nav.sessions" },
] as const;

export function AdminShell() {
	const { t } = useTranslation("admin");
	const user = useAuthStore((state) => state.user);
	const logout = useAuthStore((state) => state.logout);

	return (
		<main className="min-h-svh bg-[#7fb069] px-4 py-5 text-[#3f2a1d] sm:px-6 lg:px-8">
			<div className="mx-auto grid max-w-7xl gap-5">
				<header className="grid gap-4 border-[#5c3a21] border-4 bg-[#f4c95d] px-4 py-3 shadow-[6px_6px_0_#5c3a21]">
					<div className="flex flex-wrap items-center justify-between gap-3">
						<div>
							<p className="font-black text-[#7a4f2b] text-xs uppercase">
								BeaconCast
							</p>
							<h1 className="font-black text-2xl md:text-4xl">
								{t("dashboard.title")}
							</h1>
						</div>
						<div className="flex flex-wrap items-center gap-2">
							<Link
								to={appPaths.publicStatus}
								className="border-[#5c3a21] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21]"
							>
								Beacon
							</Link>
							<button
								type="button"
								onClick={() => void logout()}
								className="border-[#5c3a21] border-2 bg-[#f6b26b] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21]"
							>
								{t("actions.logout")}
							</button>
						</div>
					</div>
					<nav className="flex gap-2 overflow-x-auto pb-1">
						{navItems.map((item) => (
							<NavLink
								key={item.to}
								to={item.to}
								end={item.to === appPaths.admin}
								className={({ isActive }) =>
									`whitespace-nowrap border-[#5c3a21] border-2 px-3 py-2 font-black text-sm shadow-[2px_2px_0_#5c3a21] ${
										isActive ? "bg-[#d4ecaf]" : "bg-[#fff8db]"
									}`
								}
							>
								{t(item.labelKey)}
							</NavLink>
						))}
					</nav>
				</header>

				<section className="grid gap-5 lg:grid-cols-[260px_minmax(0,1fr)]">
					<div className="grid content-start gap-5">
						<AdminPanel title={t("session.user")}>
							<dl className="mt-3 grid gap-2 text-sm">
								<div className="flex justify-between gap-3">
									<dt className="font-bold text-[#6f543c]">ID</dt>
									<dd className="font-black">{user?.id ?? "-"}</dd>
								</div>
								<div className="flex justify-between gap-3">
									<dt className="font-bold text-[#6f543c]">
										{t("auth.username")}
									</dt>
									<dd className="font-black">{user?.username ?? "-"}</dd>
								</div>
								<div className="flex justify-between gap-3">
									<dt className="font-bold text-[#6f543c]">
										{t("auth.displayName")}
									</dt>
									<dd className="font-black">{user?.display_name ?? "-"}</dd>
								</div>
							</dl>
						</AdminPanel>
					</div>

					<div className="grid gap-5">
						<Outlet />
					</div>
				</section>
			</div>
		</main>
	);
}
