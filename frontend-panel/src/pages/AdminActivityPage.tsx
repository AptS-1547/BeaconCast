import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { formatAdminTime } from "@/features/admin/adminModel";
import { UsagePanel } from "@/features/admin/UsagePanel";
import type {
	AdminActivityEvent,
	AdminUsageSpan,
	AdminUsageSummary,
} from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";

export function AdminActivityPage() {
	const { i18n, t } = useTranslation("admin");
	const [events, setEvents] = useState<AdminActivityEvent[]>([]);
	const [spans, setSpans] = useState<AdminUsageSpan[]>([]);
	const [summary, setSummary] = useState<AdminUsageSummary | null>(null);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const [eventsPage, spansPage, summaryData] = await Promise.all([
				adminBeaconService.events(30),
				adminBeaconService.usageSpans(30),
				adminBeaconService.usageSummary(1),
			]);
			setEvents(eventsPage.items);
			setSpans(spansPage.items);
			setSummary(summaryData);
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
			setEvents([]);
			setSpans([]);
			setSummary(null);
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
				<h2 className="font-black text-2xl">{t("nav.activity")}</h2>
				<button
					type="button"
					onClick={() => void load()}
					disabled={loading}
					className="border-[#5c3a21] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
				>
					{t("actions.refresh")}
				</button>
			</div>
			{error ? (
				<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}
			<UsagePanel language={i18n.language} spans={spans} summary={summary} />
			<AdminPanel title={t("dashboard.events")}>
				{events.length > 0 ? (
					<div className="mt-3 overflow-x-auto">
						<table className="w-full min-w-[840px] border-separate border-spacing-0 text-left text-sm">
							<thead>
								<tr className="bg-[#f0d18a]">
									<th className="border-[#c99c55] border-2 p-2">
										{t("fields.createdAt")}
									</th>
									<th className="border-[#c99c55] border-2 p-2">
										{t("fields.status")}
									</th>
									<th className="border-[#c99c55] border-2 p-2">
										{t("fields.activity")}
									</th>
									<th className="border-[#c99c55] border-2 p-2">
										{t("usage.app")}
									</th>
									<th className="border-[#c99c55] border-2 p-2">
										{t("fields.project")}
									</th>
									<th className="border-[#c99c55] border-2 p-2">
										{t("fields.source")}
									</th>
								</tr>
							</thead>
							<tbody>
								{events.map((event) => (
									<tr key={event.id}>
										<td className="border-[#e0bd6a] border-2 p-2 font-mono">
											{formatAdminTime(event.created_at, i18n.language)}
										</td>
										<td className="border-[#e0bd6a] border-2 p-2">
											{event.status}
										</td>
										<td className="border-[#e0bd6a] border-2 p-2">
											<strong>
												{event.action_public_label ?? event.activity_kind}
											</strong>
										</td>
										<td className="border-[#e0bd6a] border-2 p-2">
											{event.application_label ?? event.app_label ?? "-"}
										</td>
										<td className="border-[#e0bd6a] border-2 p-2">
											{event.project_label ?? "-"}
										</td>
										<td className="border-[#e0bd6a] border-2 p-2">
											{event.source}
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				) : (
					<p className="mt-3 text-[#6f543c]">{t("empty.events")}</p>
				)}
			</AdminPanel>
		</>
	);
}
