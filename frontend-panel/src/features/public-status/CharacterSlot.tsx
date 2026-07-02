import { useTranslation } from "react-i18next";

export function CharacterSlot() {
	const { t } = useTranslation("publicStatus");

	return (
		<div
			role="img"
			className="grid aspect-square place-items-center border-[#7a4f2b] border-4 bg-[#b9d98a] shadow-[inset_0_0_0_6px_#8fbd70]"
			aria-label={t("characterSlot.ariaLabel")}
		>
			<div className="grid size-24 place-items-center border-[#5c3a21] border-4 bg-[#f6b26b] shadow-[4px_4px_0_#5c3a21]">
				<span className="font-black text-4xl" aria-hidden="true">
					BC
				</span>
			</div>
		</div>
	);
}
