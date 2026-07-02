import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { formatAdminTime, formatDuration } from "@/features/admin/adminModel";
import { appPaths } from "@/routes/routePaths";
import type {
	AdminActivityEvent,
	AdminUsageSummary,
	BeaconDevice,
	ManualOverride,
} from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";

export function AdminPage() {
	const { i18n, t } = useTranslation("admin");
	const [devices, setDevices] = useState<BeaconDevice[]>([]);
	const [events, setEvents] = useState<AdminActivityEvent[]>([]);
	const [manualOverride, setManualOverride] = useState<ManualOverride | null>(
		null,
	);
	const [usageSummary, setUsageSummary] = useState<AdminUsageSummary | null>(
		null,
	);
	const [loading, setLoading] = useState(true);
	const [loadFailed, setLoadFailed] = useState(false);

	const load = useCallback(async () => {
		setLoading(true);
		setLoadFailed(false);
		try {
			const [deviceData, eventPage, overrideData, summaryData] =
				await Promise.all([
					adminBeaconService.devices(),
					adminBeaconService.events(6),
					adminBeaconService.manualOverride(),
					adminBeaconService.usageSummary(1),
				]);
			setDevices(deviceData);
			setEvents(eventPage.items);
			setManualOverride(overrideData);
			setUsageSummary(summaryData);
		} catch {
			setLoadFailed(true);
			setDevices([]);
			setEvents([]);
			setManualOverride(null);
			setUsageSummary(null);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
	}, [load]);

	return (
		<>
			<div className="flex flex-wrap items-center justify-between gap-3">
				<div>
					<h2 className="font-black text-2xl">{t("nav.overview")}</h2>
					<p className="mt-1 font-bold text-[#6f543c] text-sm">
						{t("overview.subtitle")}
					</p>
				</div>
				<button
					type="button"
					onClick={() => void load()}
					disabled={loading}
					className="border-[#5c3a21] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
				>
					{t("actions.refresh")}
				</button>
			</div>

			{loadFailed ? (
				<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28] shadow-[3px_3px_0_#5c3a21]">
					{t("loadFailed")}
				</p>
			) : null}

			<div className="grid gap-4 md:grid-cols-3">
				<OverviewTile
					label={t("dashboard.devices")}
					to={appPaths.adminDevices}
					value={String(devices.length)}
				/>
				<OverviewTile
					label={t("usage.todayTotal")}
					to={appPaths.adminActivity}
					value={formatDuration(
						usageSummary?.total_seconds ?? 0,
						i18n.language,
					)}
				/>
				<OverviewTile
					label={t("dashboard.manualOverride")}
					to={appPaths.adminSettings}
					value={
						manualOverride?.active ? (manualOverride.activity ?? "-") : "-"
					}
				/>
			</div>

			<AdminPanel title={t("dashboard.events")}>
				{events.length > 0 ? (
					<div className="mt-3 grid gap-2">
						{events.map((event) => (
							<Link
								className="grid gap-2 border-[#c99c55] border-2 bg-[#f7e3a3] p-3 text-sm md:grid-cols-[150px_120px_minmax(0,1fr)_160px]"
								key={event.id}
								to={appPaths.adminActivity}
							>
								<span className="font-mono">
									{formatAdminTime(event.created_at, i18n.language)}
								</span>
								<strong>{event.status}</strong>
								<span>{event.action_public_label ?? event.activity_kind}</span>
								<span>
									{event.project_label ??
										event.application_label ??
										event.app_label ??
										"-"}
								</span>
							</Link>
						))}
					</div>
				) : (
					<p className="mt-3 text-[#6f543c]">{t("empty.events")}</p>
				)}
			</AdminPanel>
		</>
	);
}

function OverviewTile({
	label,
	to,
	value,
}: {
	label: string;
	to: string;
	value: string;
}) {
	return (
		<Link
			to={to}
			className="border-[#5c3a21] border-4 bg-[#fff8db] p-4 shadow-[5px_5px_0_#5c3a21]"
		>
			<p className="font-black text-[#7a4f2b] text-xs uppercase">{label}</p>
			<p className="mt-2 truncate font-black text-2xl">{value}</p>
		</Link>
	);
}
