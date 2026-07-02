import type { ActivityStatus } from "@/services/beaconService";
import {
	fallbackStatusToneClass,
	formatKeyLabel,
	statusToneClass,
} from "./publicStatusModel";

type StatusBadgeProps = {
	status: ActivityStatus;
	className?: string;
};

export function StatusBadge({ status, className = "" }: StatusBadgeProps) {
	return (
		<span
			className={`rounded-sm px-3 py-1 font-black text-sm shadow-md ${statusToneClass[status] ?? fallbackStatusToneClass} ${className}`}
		>
			{formatKeyLabel(status)}
		</span>
	);
}
