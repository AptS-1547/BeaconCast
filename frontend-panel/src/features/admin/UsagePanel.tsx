import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { formatAdminTime, formatDuration } from "@/features/admin/adminModel";
import { formatActivityKind } from "@/features/public-status/publicStatusModel";
import type {
	AdminUsageSpan,
	AdminUsageSummary,
} from "@/services/beaconService";

type UsagePanelProps = {
	language: string;
	spans: AdminUsageSpan[];
	summary: AdminUsageSummary | null;
};

export function UsagePanel({ language, spans, summary }: UsagePanelProps) {
	const { t } = useTranslation("admin");
	const maxAppSeconds = Math.max(
		1,
		...(summary?.app_totals ?? []).map((item) => item.duration_seconds),
	);

	return (
		<AdminPanel title={t("dashboard.usage")}>
			<div className="mt-3 grid gap-3 lg:grid-cols-[220px_minmax(0,1fr)]">
				<div className="border-[#c99c55] border-2 bg-[#f7e3a3] p-3">
					<p className="font-bold text-[#6f543c] text-xs uppercase">
						{t("usage.todayTotal")}
					</p>
					<p className="mt-2 font-black text-3xl">
						{formatDuration(summary?.total_seconds ?? 0, language)}
					</p>
					<p className="mt-2 text-[#6f543c] text-sm">
						{summary
							? `${formatAdminTime(summary.window_start, language)} - ${formatAdminTime(summary.window_end, language)}`
							: "-"}
					</p>
				</div>
				<div className="grid gap-3 md:grid-cols-3">
					<UsageTotals
						items={summary?.project_totals ?? []}
						language={language}
						title={t("usage.projects")}
					/>
					<UsageTotals
						items={summary?.category_totals ?? []}
						language={language}
						title={t("usage.categories")}
					/>
					<UsageTotals
						items={summary?.status_totals ?? []}
						language={language}
						title={t("usage.statuses")}
					/>
				</div>
			</div>

			<div className="mt-3 border-[#c99c55] border-2 bg-[#f7e3a3] p-3">
				<h3 className="font-black text-lg">{t("usage.apps")}</h3>
				{summary?.app_totals.length ? (
					<div className="mt-3 grid gap-2">
						{summary.app_totals.slice(0, 6).map((item) => (
							<div className="grid gap-1" key={item.key}>
								<div className="flex items-center justify-between gap-3 text-sm">
									<strong>{item.label}</strong>
									<span className="font-black">
										{formatDuration(item.duration_seconds, language)}
									</span>
								</div>
								<div className="h-3 border-[#5c3a21] border-2 bg-[#fff8db]">
									<div
										className="h-full bg-[#8bbbd9]"
										style={{
											width: `${Math.max(6, (item.duration_seconds / maxAppSeconds) * 100)}%`,
										}}
									/>
								</div>
							</div>
						))}
					</div>
				) : (
					<p className="mt-3 text-[#6f543c]">{t("empty.usage")}</p>
				)}
			</div>

			<div className="mt-3 overflow-x-auto">
				<table className="w-full min-w-[820px] border-separate border-spacing-0 text-left text-sm">
					<thead>
						<tr className="bg-[#f0d18a]">
							<th className="border-[#c99c55] border-2 p-2">
								{t("fields.createdAt")}
							</th>
							<th className="border-[#c99c55] border-2 p-2">
								{t("usage.duration")}
							</th>
							<th className="border-[#c99c55] border-2 p-2">
								{t("usage.app")}
							</th>
							<th className="border-[#c99c55] border-2 p-2">
								{t("fields.project")}
							</th>
							<th className="border-[#c99c55] border-2 p-2">
								{t("fields.activity")}
							</th>
						</tr>
					</thead>
					<tbody>
						{spans.map((span) => (
							<tr key={span.id}>
								<td className="border-[#e0bd6a] border-2 p-2 font-mono">
									{formatAdminTime(span.started_at, language)}
								</td>
								<td className="border-[#e0bd6a] border-2 p-2 font-black">
									{formatDuration(span.duration_seconds, language)}
								</td>
								<td className="border-[#e0bd6a] border-2 p-2">
									{span.application_label ?? span.app_label ?? "-"}
								</td>
								<td className="border-[#e0bd6a] border-2 p-2">
									{span.project_label ?? "-"}
								</td>
								<td className="border-[#e0bd6a] border-2 p-2">
									<strong>
										{span.action_public_label ??
											formatActivityKind(span.activity_kind)}
									</strong>
								</td>
							</tr>
						))}
					</tbody>
				</table>
				{spans.length === 0 ? (
					<p className="mt-3 text-[#6f543c]">{t("empty.usageSpans")}</p>
				) : null}
			</div>
		</AdminPanel>
	);
}

type UsageTotalsProps = {
	items: { key: string; label: string; duration_seconds: number }[];
	language: string;
	title: string;
};

function UsageTotals({ items, language, title }: UsageTotalsProps) {
	const { t } = useTranslation("admin");
	return (
		<div className="border-[#c99c55] border-2 bg-[#f7e3a3] p-3">
			<h3 className="font-black">{title}</h3>
			{items.length > 0 ? (
				<dl className="mt-2 grid gap-2">
					{items.slice(0, 4).map((item) => (
						<div className="flex justify-between gap-3" key={item.key}>
							<dt className="truncate font-bold text-[#6f543c]">
								{item.label}
							</dt>
							<dd className="font-black">
								{formatDuration(item.duration_seconds, language)}
							</dd>
						</div>
					))}
				</dl>
			) : (
				<p className="mt-2 text-[#6f543c] text-sm">{t("empty.usage")}</p>
			)}
		</div>
	);
}
