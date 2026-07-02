import type { ReactNode } from "react";

type AdminPanelProps = {
	children: ReactNode;
	className?: string;
	title?: string;
};

export function AdminPanel({
	children,
	className = "",
	title,
}: AdminPanelProps) {
	return (
		<section
			className={`border-[#5c3a21] border-4 bg-[#fff8db] p-4 shadow-[5px_5px_0_#5c3a21] ${className}`}
		>
			{title ? <h2 className="font-black text-xl">{title}</h2> : null}
			{children}
		</section>
	);
}
