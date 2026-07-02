import { useTranslation } from "react-i18next";
import type { ActivityLogEntry } from "@/services/beaconService";
import {
	formatActivityTime,
	formatPublicActivityMessage,
} from "./publicStatusModel";
import { StatusBadge } from "./StatusBadge";

type PublicLogItemProps = {
	entry: ActivityLogEntry;
	language: string;
};

export function PublicLogItem({ entry, language }: PublicLogItemProps) {
	const { t } = useTranslation("publicStatus");
	const activity = entry.activity;
	const project = activity.project?.label ?? t("common.noProject");
	const message = formatPublicActivityMessage(activity);

	return (
		<li className="relative grid gap-3 border-[#7a4f2b] border-l-4 bg-[#fff8db] px-4 py-3 shadow-[4px_4px_0_#5c3a21]">
			<span className="-left-[13px] absolute top-4 size-5 border-2 border-[#5c3a21] bg-[#f4c95d]" />
			<div className="grid gap-1">
				<strong className="text-[#3f2a1d] text-lg leading-tight">
					{message.headline}
				</strong>
				{message.subline ? (
					<span className="font-bold text-[#6f543c] text-sm">
						{message.subline}
					</span>
				) : null}
			</div>
			<div className="flex flex-wrap items-center gap-2">
				<StatusBadge className="px-2 py-0.5 text-xs" status={activity.status} />
				{activity.stale ? (
					<span className="rounded-sm bg-[#d7c788] px-2 py-0.5 font-bold text-[#493b14] text-xs">
						{t("freshness.staleShort")}
					</span>
				) : null}
			</div>
			<div className="flex flex-wrap gap-x-4 gap-y-1 text-[#6f543c] text-sm">
				<span>{formatActivityTime(activity.updated_at, language)}</span>
				<span>{project}</span>
			</div>
			{message.badges.length > 0 ? (
				<div className="flex flex-wrap gap-2">
					{message.badges.map((badge) => (
						<span
							className="border-[#c99c55] border-2 bg-[#d0e0e3] px-2 py-0.5 font-bold text-[#3f6670] text-xs"
							key={badge}
						>
							{badge}
						</span>
					))}
				</div>
			) : null}
		</li>
	);
}
