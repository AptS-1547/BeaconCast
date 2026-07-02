export function formatAdminTime(
	value: string | null | undefined,
	language: string,
) {
	if (!value) return "-";
	return new Intl.DateTimeFormat(language, {
		month: "2-digit",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
	}).format(new Date(value));
}

export function formatDuration(seconds: number, language: string) {
	if (seconds <= 0) return "0m";
	if (seconds < 60) return `${Math.round(seconds)}s`;
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	if (hours > 0) {
		return (
			new Intl.NumberFormat(language, {
				minimumIntegerDigits: 1,
			}).format(hours) +
			"h " +
			new Intl.NumberFormat(language, {
				minimumIntegerDigits: 2,
			}).format(minutes) +
			"m"
		);
	}
	return `${Math.max(1, minutes)}m`;
}
