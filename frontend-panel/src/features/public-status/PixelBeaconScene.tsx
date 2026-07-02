import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { appPaths } from "@/routes/routePaths";
import type { PublicActivity } from "@/services/beaconService";
import type { PublicActivityMessage } from "./publicStatusModel";
import { StatusBadge } from "./StatusBadge";

export type PublicScenePanel = "details" | "log" | "notice";

type PixelBeaconSceneProps = {
	activity: PublicActivity;
	badges: string[];
	displayMessage: PublicActivityMessage;
	displayName: string;
	freshness: string;
	loading: boolean;
	onOpenPanel: (panel: PublicScenePanel) => void;
	onRefresh: () => void;
	projectLabel: string;
	sourceLabel: string;
};

export function PixelBeaconScene({
	activity,
	badges,
	displayMessage,
	displayName,
	freshness,
	loading,
	onOpenPanel,
	onRefresh,
	projectLabel,
	sourceLabel,
}: PixelBeaconSceneProps) {
	const { t } = useTranslation("publicStatus");

	return (
		<section className="relative h-svh min-h-[620px] overflow-hidden bg-[#201711] text-[#3f2a1d]">
			<img
				alt=""
				aria-hidden="true"
				className="absolute inset-0 h-full w-full object-cover"
				src="/static/beaconcast-room-board-background-v1.png"
			/>
			<div className="absolute inset-0 bg-[radial-gradient(circle_at_24%_58%,transparent_0_28%,rgba(32,23,17,0.18)_52%,rgba(32,23,17,0.46)_100%)]" />
			<div className="absolute inset-x-0 bottom-0 h-36 bg-[linear-gradient(180deg,transparent,rgba(32,23,17,0.58))]" />

			<header className="absolute inset-x-0 top-0 z-20 flex flex-wrap items-start justify-between gap-3 p-3 sm:p-5">
				<div className="max-w-[min(520px,calc(100vw-2rem))] border-[#4b3323] border-4 bg-[#f4c95d]/95 px-4 py-3 shadow-[5px_5px_0_#4b3323] backdrop-blur-sm">
					<p className="font-black text-[#7a4f2b] text-xs uppercase">
						BeaconCast
					</p>
					<h1 className="font-black text-2xl leading-tight md:text-4xl">
						{t("hero.title", { displayName })}
					</h1>
				</div>
				<div className="flex flex-wrap justify-end gap-3">
					<button
						type="button"
						onClick={onRefresh}
						disabled={loading}
						className="border-[#4b3323] border-2 bg-[#fff8db]/95 px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] transition hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{loading ? t("actions.loading") : t("actions.refresh")}
					</button>
					<Link
						to={appPaths.admin}
						className="border-[#4b3323] border-2 bg-[#fff8db]/95 px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] transition hover:-translate-y-0.5 focus-visible:outline-3 focus-visible:outline-[#8bbbd9]"
					>
						{t("hero.adminMenu")}
					</Link>
				</div>
			</header>

			<div className="absolute right-[3%] bottom-[13%] left-[3%] z-10 grid gap-3 sm:right-[6%] sm:left-[6%] lg:top-[22%] lg:right-[7%] lg:bottom-auto lg:left-auto lg:w-[42%] xl:top-[23%] xl:right-[8%] xl:w-[41%]">
				<div className="border-[#4b3323] border-4 bg-[#fff8db]/90 p-4 shadow-[6px_6px_0_rgba(75,51,35,0.88)] backdrop-blur-[2px] md:p-5 xl:p-6">
					<div className="mb-3 flex flex-wrap items-center gap-2">
						<span className="border-[#7a4f2b] border-2 bg-[#f4c95d] px-2 py-1 font-black text-[#7a4f2b] text-xs uppercase">
							{t("currentTask.label")}
						</span>
						<StatusBadge status={activity.status} />
						<span className="border-[#7a4f2b] border-2 bg-[#d0e0e3] px-2 py-1 font-bold text-[#3f6670] text-xs">
							{activity.stale ? t("freshness.stale") : t("freshness.fresh")}
						</span>
					</div>
					<h2 className="text-balance font-black text-3xl leading-tight md:text-5xl xl:text-6xl">
						{displayMessage.headline}
					</h2>
					{displayMessage.subline ? (
						<p className="mt-3 max-w-2xl font-bold text-[#6f543c] text-base leading-7 md:text-lg">
							{displayMessage.subline}
						</p>
					) : null}
					{badges.length > 0 ? (
						<div className="mt-4 flex max-h-24 flex-wrap gap-2 overflow-hidden">
							{badges.slice(0, 8).map((badge) => (
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

				<div className="grid grid-cols-2 gap-2 text-sm sm:grid-cols-4 lg:grid-cols-2 xl:grid-cols-4">
					<SceneStat label={t("currentTask.project")} value={projectLabel} />
					<SceneStat label={t("currentTask.source")} value={sourceLabel} />
					<SceneStat label={t("currentTask.updated")} value={freshness} />
				</div>
			</div>

			<div className="absolute bottom-4 left-4 z-20 flex max-w-[calc(100vw-2rem)] flex-wrap gap-3">
				<SceneButton onClick={() => onOpenPanel("details")}>
					{t("panels.details")}
				</SceneButton>
				<SceneButton onClick={() => onOpenPanel("log")}>
					{t("panels.log")}
				</SceneButton>
				<SceneButton onClick={() => onOpenPanel("notice")}>
					{t("panels.notice")}
				</SceneButton>
			</div>
		</section>
	);
}

function SceneButton({
	children,
	onClick,
}: {
	children: string;
	onClick: () => void;
}) {
	return (
		<button
			type="button"
			onClick={onClick}
			className="border-[#4b3323] border-2 bg-[#fff8db]/95 px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] backdrop-blur-sm transition hover:-translate-y-0.5 focus-visible:outline-3 focus-visible:outline-[#8bbbd9]"
		>
			{children}
		</button>
	);
}

function SceneStat({ label, value }: { label: string; value: string }) {
	return (
		<div className="min-w-0 border-[#4b3323] border-2 bg-[#f0d18a]/92 px-3 py-2 shadow-[3px_3px_0_rgba(75,51,35,0.82)] backdrop-blur-sm">
			<p className="font-black text-[#7a4f2b] text-xs uppercase">{label}</p>
			<p className="mt-1 truncate font-bold text-[#3f2a1d]">{value}</p>
		</div>
	);
}
