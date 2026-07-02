import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { GamePanel } from "@/features/public-status/GamePanel";
import { PixelBeaconScene } from "@/features/public-status/PixelBeaconScene";
import { PublicLogItem } from "@/features/public-status/PublicLogItem";
import {
	fallbackNow,
	formatActivityTime,
	formatKeyLabel,
	formatPublicActivityMessage,
	formatRelativeActivityTime,
} from "@/features/public-status/publicStatusModel";
import type {
	ActivityLogEntry,
	ActivitySummary,
	NowResponse,
} from "@/services/beaconService";
import { publicBeaconService } from "@/services/beaconService";

function projectLabel(activity: NowResponse["now"], noProjectLabel: string) {
	return activity.project?.label ?? noProjectLabel;
}

export function PublicStatusPage() {
	const { i18n, t } = useTranslation("publicStatus");
	const [now, setNow] = useState<NowResponse | null>(null);
	const [summary, setSummary] = useState<ActivitySummary | null>(null);
	const [log, setLog] = useState<ActivityLogEntry[]>([]);
	const [loading, setLoading] = useState(true);
	const [loadFailed, setLoadFailed] = useState(false);

	const load = useCallback(async () => {
		setLoading(true);
		setLoadFailed(false);
		try {
			const [nowResponse, summaryResponse, logResponse] = await Promise.all([
				publicBeaconService.now(),
				publicBeaconService.summary(),
				publicBeaconService.activityLog(10),
			]);
			setNow(nowResponse);
			setSummary(summaryResponse);
			setLog(logResponse.items);
		} catch {
			setNow(fallbackNow);
			setSummary(null);
			setLog([]);
			setLoadFailed(true);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
		const interval = window.setInterval(() => void load(), 30_000);
		return () => window.clearInterval(interval);
	}, [load]);

	const activity = (now ?? fallbackNow).now;
	const noProjectLabel = t("common.noProject");
	const currentProjectLabel = projectLabel(activity, noProjectLabel);
	const displayMessage = formatPublicActivityMessage(activity);
	const freshness = useMemo(
		() => formatRelativeActivityTime(activity.updated_at, i18n.language),
		[activity.updated_at, i18n.language],
	);
	const sourceLabel = formatKeyLabel(activity.source);
	const windowLabel = summary
		? t("currentTask.windowDays", {
				count: summary.window_days,
			})
		: t("currentTask.windowDisabled");

	return (
		<main className="min-h-svh overflow-hidden bg-[#7fb069] text-[#3f2a1d]">
			<div className="relative mx-auto grid min-h-svh max-w-7xl gap-6 px-4 py-5 sm:px-6 lg:grid-cols-[minmax(0,1fr)_360px] lg:px-8">
				<div className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-[linear-gradient(180deg,transparent,#5d8d52)]" />
				<section className="relative grid content-start gap-5">
					<PixelBeaconScene
						activity={activity}
						badges={displayMessage.badges}
						displayMessage={displayMessage}
						freshness={freshness}
						loading={loading}
						onRefresh={() => void load()}
						projectLabel={currentProjectLabel}
						sourceLabel={sourceLabel}
						windowLabel={windowLabel}
					/>

					<GamePanel className="grid gap-4 p-4 md:p-6" tone="journal">
						<div className="flex flex-wrap items-end justify-between gap-3">
							<div>
								<p className="font-black text-[#7a4f2b] text-xs uppercase">
									Journal trail
								</p>
								<h2 className="font-black text-2xl">{t("log.title")}</h2>
							</div>
							<button
								type="button"
								onClick={() => void load()}
								disabled={loading}
								className="border-[#5c3a21] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] transition hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:opacity-60"
							>
								{loading ? t("actions.loading") : t("actions.refresh")}
							</button>
						</div>
						{loadFailed ? (
							<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
								{t("log.loadFailed")}
							</p>
						) : null}
						{log.length > 0 ? (
							<ol className="grid gap-4 pl-3">
								{log.map((entry) => (
									<PublicLogItem
										entry={entry}
										key={entry.public_id}
										language={i18n.language}
									/>
								))}
							</ol>
						) : (
							<div className="border-[#7a4f2b] border-2 bg-[#fff8db] p-5 text-[#6f543c]">
								<p className="font-black text-[#3f2a1d]">
									{t("emptyLog.title")}
								</p>
								<p className="mt-2 leading-7">{t("emptyLog.description")}</p>
							</div>
						)}
					</GamePanel>
				</section>

				<aside className="relative grid content-start gap-5 lg:sticky lg:top-5">
					<GamePanel className="p-4">
						<p className="font-black text-[#7a4f2b] text-xs uppercase">
							Notice board
						</p>
						<h2 className="mt-1 font-black text-2xl">{t("notice.title")}</h2>
						<dl className="mt-4 grid gap-3">
							<div className="flex justify-between gap-3 border-[#e0bd6a] border-b-2 pb-2">
								<dt className="font-bold text-[#6f543c]">
									{t("notice.totalEvents")}
								</dt>
								<dd className="font-black">
									{summary?.total_events ?? log.length}
								</dd>
							</div>
							<div className="flex justify-between gap-3 border-[#e0bd6a] border-b-2 pb-2">
								<dt className="font-bold text-[#6f543c]">
									{t("notice.currentSignal")}
								</dt>
								<dd className="font-black">
									{activity.message?.headline ?? activity.status}
								</dd>
							</div>
							<div className="flex justify-between gap-3 border-[#e0bd6a] border-b-2 pb-2">
								<dt className="font-bold text-[#6f543c]">
									{t("notice.lastUpdated")}
								</dt>
								<dd className="font-black">
									{formatActivityTime(activity.updated_at, i18n.language)}
								</dd>
							</div>
						</dl>
					</GamePanel>

					<GamePanel className="p-4" tone="notice">
						<p className="font-black text-[#4f7139] text-xs uppercase">
							Animation slot
						</p>
						<h2 className="mt-1 font-black text-xl">
							{t("characterSlot.title")}
						</h2>
						<p className="mt-3 text-[#4c633d] leading-7">
							{t("characterSlot.description")}
						</p>
					</GamePanel>
				</aside>
			</div>
		</main>
	);
}
