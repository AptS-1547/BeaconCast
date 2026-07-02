import type { NowResponse, PublicActivity } from "@/services/beaconService";

export const fallbackNow: NowResponse = {
	profile: {
		display_name: "BeaconCast",
	},
	now: {
		activity_kind: "unknown",
		status: "unknown",
		stale: true,
		updated_at: new Date().toISOString(),
		project: null,
		source: null,
		context_label: null,
		detail_badges: [],
		message: {
			headline: "Unknown",
			subline: null,
			badges: ["unknown"],
		},
	},
};

export const statusToneClass: Record<string, string> = {
	coding: "bg-[#5e9f6b] text-white shadow-[#2e5f3b]/20",
	studying: "bg-[#5b91b8] text-white shadow-[#2c5570]/20",
	writing: "bg-[#a97942] text-white shadow-[#70451f]/20",
	reading: "bg-[#8768a8] text-white shadow-[#4e3769]/20",
	resting: "bg-[#d89955] text-[#44270f] shadow-[#9c6026]/20",
	idle: "bg-[#d7c788] text-[#493b14] shadow-[#9a883d]/20",
	private: "bg-[#6f6a5b] text-white shadow-[#3d392f]/20",
	offline: "bg-[#b5534c] text-white shadow-[#79302b]/20",
	unknown: "bg-[#8a806f] text-white shadow-[#514838]/20",
};

export const fallbackStatusToneClass =
	"bg-[#8a806f] text-white shadow-[#514838]/20";

export function formatActivityTime(value: string, language: string) {
	return new Intl.DateTimeFormat(language, {
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
	}).format(new Date(value));
}

export function formatRelativeActivityTime(
	value: string,
	language: string,
	now = Date.now(),
) {
	const diffMinutes = Math.round((new Date(value).getTime() - now) / 60_000);
	const formatter = new Intl.RelativeTimeFormat(language, {
		numeric: "auto",
	});

	const absMinutes = Math.abs(diffMinutes);
	if (absMinutes < 1) {
		return formatter.format(0, "minute");
	}
	if (absMinutes < 60) {
		return formatter.format(diffMinutes, "minute");
	}

	const diffHours = Math.round(diffMinutes / 60);
	if (Math.abs(diffHours) < 24) {
		return formatter.format(diffHours, "hour");
	}

	const diffDays = Math.round(diffHours / 24);
	return formatter.format(diffDays, "day");
}

export type PublicActivityMessage = {
	headline: string;
	subline: string | null;
	badges: string[];
};

export function formatPublicActivityMessage(
	activity: PublicActivity,
): PublicActivityMessage {
	if (activity.message) {
		return {
			...activity.message,
			subline: activity.message.subline ?? null,
		};
	}
	const project = activity.project?.label ?? null;
	const activityLabel = formatActivityKind(activity.activity_kind);
	const headline = project ? `${activityLabel} - ${project}` : activityLabel;
	const subline = project ? activityLabel : null;
	const badges = uniquePublicBadges([
		formatKeyLabel(activity.status),
		project,
		activity.context_label,
		...activity.detail_badges,
	]);

	return {
		headline,
		subline,
		badges,
	};
}

export function formatActivityKind(activityKind: string) {
	return formatKeyLabel(activityKind || "unknown");
}

export function formatKeyLabel(value: string | null | undefined) {
	return value ? value.replaceAll("_", " ") : "-";
}

export function uniquePublicBadges(values: Array<string | null | undefined>) {
	return values.filter(
		(value, index, array): value is string =>
			Boolean(value) && array.indexOf(value) === index,
	);
}
