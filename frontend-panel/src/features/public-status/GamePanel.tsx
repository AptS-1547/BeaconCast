import type { ReactNode } from "react";

type GamePanelProps = {
	children: ReactNode;
	className?: string;
	tone?: "paper" | "field" | "notice" | "journal";
};

const toneClass = {
	paper: "bg-[#fff8db]",
	field: "bg-[#f7e3a3]",
	notice: "bg-[#d4ecaf]",
	journal: "bg-[#f0d18a]",
} as const;

export function GamePanel({
	children,
	className = "",
	tone = "paper",
}: GamePanelProps) {
	return (
		<section
			className={`border-[#5c3a21] border-4 ${toneClass[tone]} shadow-[6px_6px_0_#5c3a21] ${className}`}
		>
			{children}
		</section>
	);
}
