import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { formatActivityKind } from "@/features/public-status/publicStatusModel";
import {
	type AgentPolicy,
	adminBeaconService,
	type NowResponse,
	type PublicMessagePart,
	publicBeaconService,
	type UpdateAgentPolicyRequest,
	type UpdateVisibilityPolicyRequest,
	type VisibilityPolicy,
} from "@/services/beaconService";

type PolicyPanelProps = {
	agentPolicy: AgentPolicy | null;
	loading?: boolean;
	onSaved: () => Promise<void>;
	visibilityPolicy: VisibilityPolicy | null;
};

const publicMessageParts: PublicMessagePart[] = [
	"status",
	"activity",
	"project",
	"category",
	"app",
	"source",
	"browser_context",
	"context",
	"git_branch",
];

const defaultVisibilityPolicy: UpdateVisibilityPolicyRequest = {
	message_parts: [
		"status",
		"activity",
		"project",
		"context",
		"browser_context",
		"app",
		"source",
		"git_branch",
	],
	public_history_enabled: true,
	public_history_days: 7,
	public_history_limit: 10,
	private_mode_enabled: false,
	private_mode_label: "Signal hidden",
};

const defaultAgentPolicy: UpdateAgentPolicyRequest = {
	config_poll_interval_seconds: 300,
	report_interval_seconds: 30,
	include_app_label: true,
};

export function PolicyPanel({
	agentPolicy,
	loading = false,
	onSaved,
	visibilityPolicy,
}: PolicyPanelProps) {
	const { t } = useTranslation("admin");
	const [visibilityDraft, setVisibilityDraft] =
		useState<UpdateVisibilityPolicyRequest>(defaultVisibilityPolicy);
	const [agentDraft, setAgentDraft] =
		useState<UpdateAgentPolicyRequest>(defaultAgentPolicy);
	const [savingVisibility, setSavingVisibility] = useState(false);
	const [savingAgent, setSavingAgent] = useState(false);
	const [error, setError] = useState<string | null>(null);
	const [preview, setPreview] = useState<NowResponse | null>(null);

	useEffect(() => {
		if (visibilityPolicy) setVisibilityDraft(visibilityPolicy);
	}, [visibilityPolicy]);

	useEffect(() => {
		if (agentPolicy) setAgentDraft(agentPolicy);
	}, [agentPolicy]);

	useEffect(() => {
		let active = true;
		publicBeaconService
			.now()
			.then((response) => {
				if (active) setPreview(response);
			})
			.catch(() => {
				if (active) setPreview(null);
			});
		return () => {
			active = false;
		};
	}, []);

	async function handleSaveVisibility(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setError(null);
		setSavingVisibility(true);
		try {
			await adminBeaconService.updateVisibilityPolicy(visibilityDraft);
			await onSaved();
		} catch (saveError) {
			setError(saveError instanceof Error ? saveError.message : "save failed");
		} finally {
			setSavingVisibility(false);
		}
	}

	async function handleSaveAgent(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setError(null);
		setSavingAgent(true);
		try {
			await adminBeaconService.updateAgentPolicy(agentDraft);
			await onSaved();
		} catch (saveError) {
			setError(saveError instanceof Error ? saveError.message : "save failed");
		} finally {
			setSavingAgent(false);
		}
	}

	const disabled = loading || savingVisibility || savingAgent;

	return (
		<AdminPanel title={t("dashboard.systemConfig")}>
			{error ? (
				<p className="mt-3 border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}
			<div className="mt-3 grid gap-4 xl:grid-cols-2">
				<form
					className="grid gap-3 border-[#c99c55] border-2 bg-[#f7e3a3] p-3"
					onSubmit={handleSaveVisibility}
				>
					<div className="flex flex-wrap items-center justify-between gap-2">
						<h3 className="font-black text-lg">{t("policy.publicBoard")}</h3>
						<span className="border-[#5c3a21] border-2 bg-[#d9ead3] px-2 py-1 font-black text-xs">
							{t("visibility.partsCount", {
								count: visibilityDraft.message_parts.length,
							})}
						</span>
					</div>
					<div className="grid gap-2 sm:grid-cols-2">
						{publicMessageParts.map((part) => (
							<ToggleField
								key={part}
								checked={visibilityDraft.message_parts.includes(part)}
								label={t(`visibility.parts.${part}`)}
								onChange={(checked) =>
									setVisibilityDraft((draft) => ({
										...draft,
										message_parts: checked
											? appendMessagePart(draft.message_parts, part)
											: draft.message_parts.filter((item) => item !== part),
									}))
								}
							/>
						))}
					</div>
					<div className="grid gap-2 sm:grid-cols-2">
						<ToggleField
							checked={visibilityDraft.public_history_enabled}
							label={t("visibility.historyEnabled")}
							onChange={(checked) =>
								setVisibilityDraft((draft) => ({
									...draft,
									public_history_enabled: checked,
								}))
							}
						/>
						<ToggleField
							checked={visibilityDraft.private_mode_enabled}
							label={t("visibility.privateMode")}
							onChange={(checked) =>
								setVisibilityDraft((draft) => ({
									...draft,
									private_mode_enabled: checked,
								}))
							}
						/>
					</div>
					<div className="grid gap-3 sm:grid-cols-[140px_140px_minmax(0,1fr)]">
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("visibility.historyDays")}</span>
							<input
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								min={1}
								type="number"
								value={visibilityDraft.public_history_days}
								onChange={(event) =>
									setVisibilityDraft((draft) => ({
										...draft,
										public_history_days: Number(event.target.value),
									}))
								}
							/>
						</label>
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("visibility.historyLimit")}</span>
							<input
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								min={1}
								max={200}
								type="number"
								value={visibilityDraft.public_history_limit}
								onChange={(event) =>
									setVisibilityDraft((draft) => ({
										...draft,
										public_history_limit: Number(event.target.value),
									}))
								}
							/>
						</label>
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("visibility.privateLabel")}</span>
							<input
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								value={visibilityDraft.private_mode_label}
								onChange={(event) =>
									setVisibilityDraft((draft) => ({
										...draft,
										private_mode_label: event.target.value,
									}))
								}
							/>
						</label>
					</div>
					<button
						type="submit"
						disabled={disabled}
						className="justify-self-start border-[#5c3a21] border-2 bg-[#f4c95d] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
					>
						{savingVisibility
							? t("actions.saving")
							: t("actions.saveVisibility")}
					</button>
				</form>

				<form
					className="grid gap-3 border-[#c99c55] border-2 bg-[#f7e3a3] p-3"
					onSubmit={handleSaveAgent}
				>
					<div className="flex flex-wrap items-center justify-between gap-2">
						<h3 className="font-black text-lg">{t("policy.agentRunner")}</h3>
						<span className="border-[#5c3a21] border-2 bg-[#d0e0e3] px-2 py-1 font-black text-xs">
							{agentDraft.report_interval_seconds}s
						</span>
					</div>
					<div className="grid gap-3 sm:grid-cols-2">
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("agent.reportInterval")}</span>
							<input
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								min={1}
								type="number"
								value={agentDraft.report_interval_seconds}
								onChange={(event) =>
									setAgentDraft((draft) => ({
										...draft,
										report_interval_seconds: Number(event.target.value),
									}))
								}
							/>
						</label>
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("agent.configPollInterval")}</span>
							<input
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								min={1}
								type="number"
								value={agentDraft.config_poll_interval_seconds}
								onChange={(event) =>
									setAgentDraft((draft) => ({
										...draft,
										config_poll_interval_seconds: Number(event.target.value),
									}))
								}
							/>
						</label>
					</div>
					<ToggleField
						checked={agentDraft.include_app_label}
						label={t("agent.includeAppLabel")}
						onChange={(checked) =>
							setAgentDraft((draft) => ({
								...draft,
								include_app_label: checked,
							}))
						}
					/>
					<button
						type="submit"
						disabled={disabled}
						className="justify-self-start border-[#5c3a21] border-2 bg-[#f4c95d] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
					>
						{savingAgent ? t("actions.saving") : t("actions.saveAgent")}
					</button>
				</form>
			</div>
			<VisibilityPreview
				messageParts={visibilityDraft.message_parts}
				preview={preview}
				privateModeEnabled={visibilityDraft.private_mode_enabled}
				privateModeLabel={visibilityDraft.private_mode_label}
			/>
		</AdminPanel>
	);
}

type VisibilityPreviewProps = {
	messageParts: PublicMessagePart[];
	preview: NowResponse | null;
	privateModeEnabled: boolean;
	privateModeLabel: string;
};

function VisibilityPreview({
	messageParts,
	preview,
	privateModeEnabled,
	privateModeLabel,
}: VisibilityPreviewProps) {
	const { t } = useTranslation("admin");
	const projected = projectPreview(
		preview,
		messageParts,
		privateModeEnabled,
		privateModeLabel,
	);
	return (
		<div className="mt-4 border-[#c99c55] border-2 bg-[#f7e3a3] p-3">
			<div className="flex flex-wrap items-center justify-between gap-2">
				<h3 className="font-black text-lg">{t("policy.preview")}</h3>
				<span className="border-[#5c3a21] border-2 bg-[#fff8db] px-2 py-1 font-black text-xs">
					{t("visibility.partsCount", { count: messageParts.length })}
				</span>
			</div>
			<div className="mt-3 grid gap-2 text-sm sm:grid-cols-2 lg:grid-cols-4">
				<PreviewCell label={t("fields.status")} value={projected.status} />
				<PreviewCell label={t("fields.activity")} value={projected.activity} />
				<PreviewCell label={t("fields.project")} value={projected.project} />
				<PreviewCell label={t("policy.details")} value={projected.details} />
			</div>
		</div>
	);
}

function PreviewCell({ label, value }: { label: string; value: string }) {
	return (
		<div className="border-[#d6ac62] border-2 bg-[#fff8db] p-2">
			<p className="font-bold text-[#6f543c] text-xs">{label}</p>
			<p className="mt-1 break-words font-black">{value || "-"}</p>
		</div>
	);
}

function projectPreview(
	preview: NowResponse | null,
	messageParts: PublicMessagePart[],
	privateModeEnabled: boolean,
	privateModeLabel: string,
) {
	if (privateModeEnabled) {
		return {
			status: "private",
			activity: privateModeLabel,
			project: "-",
			details: "-",
		};
	}
	const activity = preview?.now;
	if (!activity || messageParts.length === 0) {
		return {
			status: "-",
			activity: "Signal hidden",
			project: "-",
			details: "-",
		};
	}
	const status = messageParts.includes("status") ? activity.status : "-";
	const project = messageParts.includes("project")
		? (activity.project?.label ?? activity.project?.key ?? "-")
		: "-";
	const visibleActivity = messageParts.includes("activity")
		? activity.message?.headline || formatActivityKind(activity.activity_kind)
		: status;
	const details = [
		messageParts.includes("context") ? activity.context_label : null,
		messageParts.includes("source") ? activity.source : null,
		...(messageParts.some((part) =>
			["category", "app", "browser_context", "git_branch"].includes(part),
		)
			? activity.detail_badges
			: []),
	]
		.filter(Boolean)
		.join(" / ");
	return {
		status,
		activity: visibleActivity,
		project,
		details: details || "-",
	};
}

function appendMessagePart(
	parts: PublicMessagePart[],
	part: PublicMessagePart,
): PublicMessagePart[] {
	return parts.includes(part) ? parts : [...parts, part];
}

type ToggleFieldProps = {
	checked: boolean;
	label: string;
	onChange: (checked: boolean) => void;
};

function ToggleField({ checked, label, onChange }: ToggleFieldProps) {
	return (
		<label className="flex min-h-10 items-center justify-between gap-3 border-[#c99c55] border-2 bg-[#fff8db] px-3 py-2 font-bold text-sm">
			<span>{label}</span>
			<input
				checked={checked}
				className="h-5 w-5 accent-[#7fb069]"
				type="checkbox"
				onChange={(event) => onChange(event.target.checked)}
			/>
		</label>
	);
}
