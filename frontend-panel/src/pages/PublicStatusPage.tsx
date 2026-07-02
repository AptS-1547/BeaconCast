import type { ReactNode } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import {
	PixelBeaconScene,
	type PublicScenePanel,
} from "@/features/public-status/PixelBeaconScene";
import { PublicLogItem } from "@/features/public-status/PublicLogItem";
import {
	fallbackNow,
	formatActivityTime,
	formatKeyLabel,
	formatPublicActivityMessage,
	formatRelativeActivityTime,
} from "@/features/public-status/publicStatusModel";
import type { ActivityLogEntry, NowResponse } from "@/services/beaconService";
import { publicBeaconService } from "@/services/beaconService";

function projectLabel(activity: NowResponse["now"], noProjectLabel: string) {
	return activity.project?.label ?? noProjectLabel;
}

export function PublicStatusPage() {
	const { i18n, t } = useTranslation("publicStatus");
	const [now, setNow] = useState<NowResponse | null>(null);
	const [log, setLog] = useState<ActivityLogEntry[]>([]);
	const [loading, setLoading] = useState(true);
	const [loadFailed, setLoadFailed] = useState(false);
	const [activePanel, setActivePanel] = useState<PublicScenePanel | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setLoadFailed(false);
		try {
			const [nowResponse, logResponse] = await Promise.all([
				publicBeaconService.now(),
				publicBeaconService.activityLog(200),
			]);
			setNow(nowResponse);
			setLog(logResponse.items);
		} catch {
			setNow(fallbackNow);
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
	const displayName = (now ?? fallbackNow).profile.display_name;
	const noProjectLabel = t("common.noProject");
	const currentProjectLabel = projectLabel(activity, noProjectLabel);
	const displayMessage = formatPublicActivityMessage(activity);
	const freshness = useMemo(
		() => formatRelativeActivityTime(activity.updated_at, i18n.language),
		[activity.updated_at, i18n.language],
	);
	const sourceLabel = formatKeyLabel(activity.source);

	return (
		<main className="h-svh overflow-hidden bg-[#201711] text-[#3f2a1d]">
			<PixelBeaconScene
				activity={activity}
				badges={displayMessage.badges}
				displayMessage={displayMessage}
				displayName={displayName}
				freshness={freshness}
				loading={loading}
				onOpenPanel={setActivePanel}
				onRefresh={() => void load()}
				projectLabel={currentProjectLabel}
				sourceLabel={sourceLabel}
			/>

			{activePanel ? (
				<SceneDialog
					title={t(`panels.${activePanel}`)}
					onClose={() => setActivePanel(null)}
				>
					{activePanel === "details" ? (
						<DetailsPanel
							activity={activity}
							badges={displayMessage.badges}
							freshness={freshness}
							language={i18n.language}
							projectLabel={currentProjectLabel}
							sourceLabel={sourceLabel}
						/>
					) : null}
					{activePanel === "log" ? (
						<LogPanel
							language={i18n.language}
							loadFailed={loadFailed}
							log={log}
						/>
					) : null}
					{activePanel === "notice" ? (
						<NoticePanel activity={activity} language={i18n.language} />
					) : null}
				</SceneDialog>
			) : null}
		</main>
	);
}

function SceneDialog({
	children,
	onClose,
	title,
}: {
	children: ReactNode;
	onClose: () => void;
	title: string;
}) {
	const { t } = useTranslation("publicStatus");

	return (
		<div className="fixed inset-0 z-50 grid place-items-center bg-[#201711]/55 p-4 backdrop-blur-[2px]">
			<button
				type="button"
				className="absolute inset-0 cursor-default"
				aria-label={t("actions.close")}
				onClick={onClose}
			/>
			<section
				aria-modal="true"
				role="dialog"
				className="relative z-10 grid max-h-[min(760px,86svh)] w-full max-w-2xl grid-rows-[auto_minmax(0,1fr)] overflow-hidden border-[#4b3323] border-4 bg-[#fff8db] shadow-[8px_8px_0_rgba(75,51,35,0.62)]"
			>
				<header className="flex items-center justify-between gap-3 border-[#d6ac62] border-b-4 bg-[#f4c95d] px-4 py-3">
					<h2 className="font-black text-2xl">{title}</h2>
					<button
						type="button"
						onClick={onClose}
						className="border-[#4b3323] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] transition hover:-translate-y-0.5 focus-visible:outline-3 focus-visible:outline-[#8bbbd9]"
					>
						{t("actions.close")}
					</button>
				</header>
				<div className="min-h-0 overflow-y-auto p-4">{children}</div>
			</section>
		</div>
	);
}

function DetailsPanel({
	activity,
	badges,
	freshness,
	language,
	projectLabel,
	sourceLabel,
}: {
	activity: NowResponse["now"];
	badges: string[];
	freshness: string;
	language: string;
	projectLabel: string;
	sourceLabel: string;
}) {
	const { t } = useTranslation("publicStatus");

	return (
		<div className="grid gap-4">
			<InfoGrid>
				<InfoRow label={t("currentTask.project")} value={projectLabel} />
				<InfoRow label={t("currentTask.source")} value={sourceLabel} />
				<InfoRow label={t("currentTask.updated")} value={freshness} />
				<InfoRow label={t("notice.currentStatus")} value={activity.status} />
				<InfoRow
					label={t("notice.lastUpdated")}
					value={formatActivityTime(activity.updated_at, language)}
				/>
			</InfoGrid>
			{badges.length > 0 ? (
				<div className="flex flex-wrap gap-2">
					{badges.map((badge) => (
						<span
							className="border-[#5c3a21] border-2 bg-[#d4ecaf] px-2 py-1 font-black text-[#3f2a1d] text-xs"
							key={badge}
						>
							{badge}
						</span>
					))}
				</div>
			) : null}
		</div>
	);
}

function LogPanel({
	language,
	loadFailed,
	log,
}: {
	language: string;
	loadFailed: boolean;
	log: ActivityLogEntry[];
}) {
	const { t } = useTranslation("publicStatus");

	return (
		<div className="grid gap-4">
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
							language={language}
						/>
					))}
				</ol>
			) : (
				<div className="border-[#7a4f2b] border-2 bg-[#fff8db] p-5 text-[#6f543c]">
					<p className="font-black text-[#3f2a1d]">{t("emptyLog.title")}</p>
					<p className="mt-2 leading-7">{t("emptyLog.description")}</p>
				</div>
			)}
		</div>
	);
}

function NoticePanel({
	activity,
	language,
}: {
	activity: NowResponse["now"];
	language: string;
}) {
	const { t } = useTranslation("publicStatus");

	return (
		<InfoGrid>
			<InfoRow
				label={t("notice.currentSignal")}
				value={activity.message?.headline ?? activity.status}
			/>
			<InfoRow
				label={t("notice.currentStatus")}
				value={formatKeyLabel(activity.status)}
			/>
			<InfoRow
				label={t("notice.lastUpdated")}
				value={formatActivityTime(activity.updated_at, language)}
			/>
		</InfoGrid>
	);
}

function InfoGrid({ children }: { children: ReactNode }) {
	return <dl className="grid gap-3">{children}</dl>;
}

function InfoRow({ label, value }: { label: string; value: string }) {
	return (
		<div className="flex justify-between gap-3 border-[#e0bd6a] border-b-2 pb-2">
			<dt className="font-bold text-[#6f543c]">{label}</dt>
			<dd className="min-w-0 max-w-[58%] truncate text-right font-black">
				{value}
			</dd>
		</div>
	);
}
