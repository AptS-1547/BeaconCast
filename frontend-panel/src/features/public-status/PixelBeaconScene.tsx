import { useTranslation } from "react-i18next";
import { Link } from "react-router-dom";
import { appPaths } from "@/routes/routePaths";
import type { PublicActivity } from "@/services/beaconService";
import type { PublicActivityMessage } from "./publicStatusModel";
import { StatusBadge } from "./StatusBadge";

type PixelBeaconSceneProps = {
	activity: PublicActivity;
	badges: string[];
	displayMessage: PublicActivityMessage;
	freshness: string;
	loading: boolean;
	onRefresh: () => void;
	projectLabel: string;
	sourceLabel: string;
	windowLabel: string;
};

export function PixelBeaconScene({
	activity,
	badges,
	displayMessage,
	freshness,
	loading,
	onRefresh,
	projectLabel,
	sourceLabel,
	windowLabel,
}: PixelBeaconSceneProps) {
	const { t } = useTranslation("publicStatus");

	return (
		<section className="relative min-h-[520px] overflow-hidden border-[#4b3323] border-4 bg-[#77b8d8] shadow-[8px_8px_0_#4b3323] md:min-h-[600px]">
			<div className="absolute inset-0 bg-[linear-gradient(180deg,#78c3e2_0%,#b7d68a_54%,#5f9d55_54%,#4f7d42_100%)]" />
			<div className="absolute right-[8%] top-10 size-20 border-[#f5d56a] border-4 bg-[#ffe27a] shadow-[0_0_0_8px_#f6b85e]" />
			<div className="absolute top-20 left-[10%] h-8 w-28 bg-[#eaf6db] shadow-[32px_0_0_#eaf6db,64px_8px_0_#eaf6db]" />
			<div className="absolute top-28 right-[22%] h-7 w-24 bg-[#dff0d0] shadow-[28px_0_0_#dff0d0,52px_-6px_0_#dff0d0]" />

			<div className="absolute inset-x-0 bottom-[150px] h-24 bg-[#7fb069] shadow-[0_-16px_0_#91c36c]" />
			<div className="absolute inset-x-0 bottom-0 h-[150px] border-[#4b3323] border-t-4 bg-[linear-gradient(90deg,#7a4f2b_0_16px,#8a5a35_16px_32px)] bg-[length:32px_32px]" />
			<div className="absolute bottom-[132px] left-[3%] h-12 w-28 border-[#4b3323] border-4 bg-[#9f6f46] shadow-[6px_6px_0_#4b3323]" />
			<div className="absolute right-[6%] bottom-[132px] h-20 w-16 border-[#4b3323] border-4 bg-[#d8a85d] shadow-[6px_6px_0_#4b3323]">
				<div className="mx-auto mt-3 h-5 w-8 bg-[#fff8db]" />
			</div>

			<div className="relative z-10 grid min-h-[520px] content-between gap-6 p-4 md:min-h-[600px] md:p-6">
				<header className="flex flex-wrap items-center justify-between gap-3">
					<div className="border-[#4b3323] border-4 bg-[#f4c95d] px-4 py-3 shadow-[5px_5px_0_#4b3323]">
						<p className="font-black text-[#7a4f2b] text-xs uppercase">
							BeaconCast
						</p>
						<h1 className="font-black text-2xl leading-tight md:text-4xl">
							{t("hero.title")}
						</h1>
					</div>
					<div className="flex flex-wrap gap-3">
						<button
							type="button"
							onClick={onRefresh}
							disabled={loading}
							className="border-[#4b3323] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] transition hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:opacity-60"
						>
							{loading ? t("actions.loading") : t("actions.refresh")}
						</button>
						<Link
							to={appPaths.admin}
							className="border-[#4b3323] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#4b3323] transition hover:-translate-y-0.5 focus-visible:outline-3 focus-visible:outline-[#2f6f8f]"
						>
							{t("hero.adminMenu")}
						</Link>
					</div>
				</header>

				<div className="grid items-end gap-5 lg:grid-cols-[minmax(0,1fr)_220px]">
					<div className="relative min-h-[300px]">
						<BeaconAvatar status={activity.status} stale={activity.stale} />
						<div className="absolute right-0 bottom-24 left-0 max-w-3xl border-[#4b3323] border-4 bg-[#fff8db] p-4 shadow-[6px_6px_0_#4b3323] md:left-36 md:p-5">
							<div className="mb-3 flex flex-wrap items-center gap-2">
								<span className="border-[#7a4f2b] border-2 bg-[#f4c95d] px-2 py-1 font-black text-[#7a4f2b] text-xs uppercase">
									{t("currentTask.label")}
								</span>
								<StatusBadge status={activity.status} />
								<span className="border-[#7a4f2b] border-2 bg-[#d0e0e3] px-2 py-1 font-bold text-[#3f6670] text-xs">
									{activity.stale ? t("freshness.stale") : t("freshness.fresh")}
								</span>
							</div>
							<h2 className="text-balance font-black text-3xl leading-tight md:text-5xl">
								{displayMessage.headline}
							</h2>
							{displayMessage.subline ? (
								<p className="mt-3 max-w-2xl font-bold text-[#6f543c] text-base leading-7 md:text-lg">
									{displayMessage.subline}
								</p>
							) : null}
							{badges.length > 0 ? (
								<div className="mt-4 flex flex-wrap gap-2">
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
					</div>

					<div className="grid gap-3 border-[#4b3323] border-4 bg-[#f0d18a] p-3 shadow-[5px_5px_0_#4b3323]">
						<SceneStat label={t("currentTask.project")} value={projectLabel} />
						<SceneStat label={t("currentTask.source")} value={sourceLabel} />
						<SceneStat label={t("currentTask.updated")} value={freshness} />
						<SceneStat label={t("currentTask.window")} value={windowLabel} />
					</div>
				</div>
			</div>
		</section>
	);
}

function BeaconAvatar({ status, stale }: { status: string; stale: boolean }) {
	const bodyColor = stale
		? "bg-[#8a806f]"
		: status === "coding"
			? "bg-[#5e9f6b]"
			: status === "reading" || status === "studying"
				? "bg-[#5b91b8]"
				: status === "offline"
					? "bg-[#b5534c]"
					: "bg-[#f6b26b]";

	return (
		<div
			className="absolute bottom-[58px] left-8 grid h-44 w-28 place-items-center md:left-14"
			aria-hidden="true"
		>
			<div className="relative h-36 w-24">
				<div className="absolute top-0 left-7 size-10 border-[#4b3323] border-4 bg-[#f6d0a0]" />
				<div
					className={`absolute top-10 left-4 h-16 w-16 border-[#4b3323] border-4 shadow-[4px_4px_0_#4b3323] ${bodyColor}`}
				/>
				<div className="absolute top-3 left-9 size-2 bg-[#4b3323] shadow-[18px_0_0_#4b3323]" />
				<div className="absolute top-24 left-2 h-10 w-6 border-[#4b3323] border-4 bg-[#3f6670]" />
				<div className="absolute top-24 right-2 h-10 w-6 border-[#4b3323] border-4 bg-[#3f6670]" />
				<div className="absolute top-[70px] -left-2 h-8 w-5 border-[#4b3323] border-4 bg-[#f6d0a0]" />
				<div className="absolute top-[70px] -right-2 h-8 w-5 border-[#4b3323] border-4 bg-[#f6d0a0]" />
			</div>
		</div>
	);
}

function SceneStat({ label, value }: { label: string; value: string }) {
	return (
		<div className="min-w-0 border-[#c99c55] border-b-2 pb-2 last:border-b-0 last:pb-0">
			<p className="font-black text-[#7a4f2b] text-xs uppercase">{label}</p>
			<p className="mt-1 truncate font-bold text-[#3f2a1d]">{value}</p>
		</div>
	);
}
