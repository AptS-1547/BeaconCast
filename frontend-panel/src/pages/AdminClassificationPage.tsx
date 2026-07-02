import type { FormEvent } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import type {
	ActivityAction,
	ActivityApplication,
	UpsertActivityActionRequest,
	UpsertActivityApplicationRequest,
} from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";

type ActionDraft = UpsertActivityActionRequest & {
	action_key: string;
};

type ApplicationDraft = UpsertActivityApplicationRequest & {
	app_key: string;
	aliasesText: string;
};

const emptyActionDraft: ActionDraft = {
	action_key: "",
	label: "",
	status: "working",
	category: "activity",
	public_label: "",
	message_template: "{action}",
	enabled: true,
	sort_order: 100,
};

const emptyApplicationDraft: ApplicationDraft = {
	app_key: "",
	display_name: "",
	default_action_key: null,
	enabled: true,
	aliases: [],
	aliasesText: "",
};

export function AdminClassificationPage() {
	const { t } = useTranslation("admin");
	const [actions, setActions] = useState<ActivityAction[]>([]);
	const [applications, setApplications] = useState<ActivityApplication[]>([]);
	const [actionDraft, setActionDraft] = useState<ActionDraft>(emptyActionDraft);
	const [applicationDraft, setApplicationDraft] = useState<ApplicationDraft>(
		emptyApplicationDraft,
	);
	const [loading, setLoading] = useState(true);
	const [saving, setSaving] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const actionOptions = useMemo(
		() =>
			actions.map((action) => ({
				key: action.action_key,
				label: `${action.public_label} (${action.action_key})`,
			})),
		[actions],
	);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const [actionData, applicationData] = await Promise.all([
				adminBeaconService.activityActions(),
				adminBeaconService.activityApplications(),
			]);
			setActions(actionData);
			setApplications(applicationData);
			setActionDraft(
				actionData[0] ? actionToDraft(actionData[0]) : emptyActionDraft,
			);
			setApplicationDraft(
				applicationData[0]
					? applicationToDraft(applicationData[0])
					: emptyApplicationDraft,
			);
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
			setActions([]);
			setApplications([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
	}, [load]);

	async function handleSaveAction(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setSaving(true);
		setError(null);
		try {
			const { action_key, ...payload } = normalizeActionDraft(actionDraft);
			await adminBeaconService.upsertActivityAction(action_key, payload);
			await load();
		} catch (saveError) {
			setError(saveError instanceof Error ? saveError.message : "save failed");
		} finally {
			setSaving(false);
		}
	}

	async function handleSaveApplication(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setSaving(true);
		setError(null);
		try {
			const { app_key, aliasesText, ...payload } =
				normalizeApplicationDraft(applicationDraft);
			void aliasesText;
			await adminBeaconService.upsertActivityApplication(app_key, payload);
			await load();
		} catch (saveError) {
			setError(saveError instanceof Error ? saveError.message : "save failed");
		} finally {
			setSaving(false);
		}
	}

	return (
		<>
			<div className="flex flex-wrap items-center justify-between gap-3">
				<div>
					<h2 className="font-black text-2xl">{t("classification.title")}</h2>
					<p className="mt-1 font-bold text-[#6f543c] text-sm">
						{t("classification.subtitle")}
					</p>
				</div>
				<button
					type="button"
					onClick={() => void load()}
					disabled={loading || saving}
					className="border-[#5c3a21] border-2 bg-[#fff8db] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
				>
					{t("actions.refresh")}
				</button>
			</div>

			{error ? (
				<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}

			<div className="grid gap-5 xl:grid-cols-2">
				<AdminPanel title={t("classification.actions")}>
					<div className="mt-3 flex flex-wrap gap-2">
						{actions.map((action) => (
							<button
								type="button"
								className="border-[#5c3a21] border-2 bg-[#fff8db] px-2 py-1 font-black text-xs"
								key={action.id}
								onClick={() => setActionDraft(actionToDraft(action))}
							>
								{action.public_label}
							</button>
						))}
						<button
							type="button"
							className="border-[#5c3a21] border-2 bg-[#d4ecaf] px-2 py-1 font-black text-xs"
							onClick={() => setActionDraft(emptyActionDraft)}
						>
							{t("classification.newAction")}
						</button>
					</div>
					<form className="mt-3 grid gap-3" onSubmit={handleSaveAction}>
						<div className="grid gap-3 sm:grid-cols-2">
							<TextField
								label={t("classification.actionKey")}
								value={actionDraft.action_key}
								onChange={(value) =>
									setActionDraft((draft) => ({
										...draft,
										action_key: toKey(value),
									}))
								}
							/>
							<TextField
								label={t("classification.label")}
								value={actionDraft.label}
								onChange={(value) =>
									setActionDraft((draft) => ({ ...draft, label: value }))
								}
							/>
							<TextField
								label={t("fields.status")}
								value={actionDraft.status}
								onChange={(value) =>
									setActionDraft((draft) => ({
										...draft,
										status: toKey(value),
									}))
								}
							/>
							<TextField
								label={t("classification.category")}
								value={actionDraft.category}
								onChange={(value) =>
									setActionDraft((draft) => ({
										...draft,
										category: toKey(value),
									}))
								}
							/>
							<TextField
								label={t("classification.publicLabel")}
								value={actionDraft.public_label}
								onChange={(value) =>
									setActionDraft((draft) => ({
										...draft,
										public_label: value,
									}))
								}
							/>
						</div>
						<TextField
							label={t("classification.messageTemplate")}
							value={actionDraft.message_template}
							onChange={(value) =>
								setActionDraft((draft) => ({
									...draft,
									message_template: value,
								}))
							}
						/>
						<div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_140px]">
							<ToggleField
								checked={actionDraft.enabled}
								label={t("classification.enabled")}
								onChange={(checked) =>
									setActionDraft((draft) => ({
										...draft,
										enabled: checked,
									}))
								}
							/>
							<label className="grid gap-1 font-bold text-sm">
								<span>{t("classification.sortOrder")}</span>
								<input
									className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
									type="number"
									value={actionDraft.sort_order}
									onChange={(event) =>
										setActionDraft((draft) => ({
											...draft,
											sort_order: Number(event.target.value),
										}))
									}
								/>
							</label>
						</div>
						<SaveButton disabled={loading || saving} />
					</form>
				</AdminPanel>

				<AdminPanel title={t("classification.applications")}>
					<div className="mt-3 flex flex-wrap gap-2">
						{applications.map((app) => (
							<button
								type="button"
								className="border-[#5c3a21] border-2 bg-[#fff8db] px-2 py-1 font-black text-xs"
								key={app.id}
								onClick={() => setApplicationDraft(applicationToDraft(app))}
							>
								{app.display_name}
							</button>
						))}
						<button
							type="button"
							className="border-[#5c3a21] border-2 bg-[#d4ecaf] px-2 py-1 font-black text-xs"
							onClick={() => setApplicationDraft(emptyApplicationDraft)}
						>
							{t("classification.newApplication")}
						</button>
					</div>
					<form className="mt-3 grid gap-3" onSubmit={handleSaveApplication}>
						<div className="grid gap-3 sm:grid-cols-2">
							<TextField
								label={t("classification.appKey")}
								value={applicationDraft.app_key}
								onChange={(value) =>
									setApplicationDraft((draft) => ({
										...draft,
										app_key: toKey(value),
									}))
								}
							/>
							<TextField
								label={t("fields.displayName")}
								value={applicationDraft.display_name}
								onChange={(value) =>
									setApplicationDraft((draft) => ({
										...draft,
										display_name: value,
									}))
								}
							/>
						</div>
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("classification.defaultAction")}</span>
							<select
								className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								value={applicationDraft.default_action_key ?? ""}
								onChange={(event) =>
									setApplicationDraft((draft) => ({
										...draft,
										default_action_key: event.target.value || null,
									}))
								}
							>
								<option value="">-</option>
								{actionOptions.map((action) => (
									<option key={action.key} value={action.key}>
										{action.label}
									</option>
								))}
							</select>
						</label>
						<label className="grid gap-1 font-bold text-sm">
							<span>{t("classification.aliases")}</span>
							<textarea
								className="min-h-28 border-[#5c3a21] border-2 bg-white px-3 py-2 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								value={applicationDraft.aliasesText}
								onChange={(event) =>
									setApplicationDraft((draft) => ({
										...draft,
										aliasesText: event.target.value,
									}))
								}
							/>
						</label>
						<ToggleField
							checked={applicationDraft.enabled}
							label={t("classification.enabled")}
							onChange={(checked) =>
								setApplicationDraft((draft) => ({
									...draft,
									enabled: checked,
								}))
							}
						/>
						<SaveButton disabled={loading || saving} />
					</form>
				</AdminPanel>
			</div>
		</>
	);
}

function actionToDraft(action: ActivityAction): ActionDraft {
	return {
		action_key: action.action_key,
		label: action.label,
		status: action.status,
		category: action.category,
		public_label: action.public_label,
		message_template: action.message_template,
		enabled: action.enabled,
		sort_order: action.sort_order,
	};
}

function applicationToDraft(app: ActivityApplication): ApplicationDraft {
	return {
		app_key: app.app_key,
		display_name: app.display_name,
		default_action_key: app.default_action_key ?? null,
		enabled: app.enabled,
		aliases: app.aliases,
		aliasesText: app.aliases.join("\n"),
	};
}

function normalizeActionDraft(draft: ActionDraft): ActionDraft {
	return {
		...draft,
		action_key: toKey(draft.action_key),
		label: draft.label.trim(),
		status: toKey(draft.status),
		category: toKey(draft.category),
		public_label: draft.public_label.trim(),
		message_template: draft.message_template.trim() || "{action}",
	};
}

function normalizeApplicationDraft(draft: ApplicationDraft): ApplicationDraft {
	return {
		...draft,
		app_key: toKey(draft.app_key),
		display_name: draft.display_name.trim(),
		default_action_key: draft.default_action_key || null,
		aliases: draft.aliasesText
			.split(/\r?\n|,/)
			.map((alias) => alias.trim())
			.filter(Boolean),
	};
}

function toKey(value: string) {
	return value
		.trim()
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "_")
		.replace(/^_+|_+$/g, "");
}

function TextField({
	label,
	onChange,
	value,
}: {
	label: string;
	onChange: (value: string) => void;
	value: string;
}) {
	return (
		<label className="grid gap-1 font-bold text-sm">
			<span>{label}</span>
			<input
				className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
				value={value}
				onChange={(event) => onChange(event.target.value)}
				required
			/>
		</label>
	);
}

function ToggleField({
	checked,
	label,
	onChange,
}: {
	checked: boolean;
	label: string;
	onChange: (checked: boolean) => void;
}) {
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

function SaveButton({ disabled }: { disabled: boolean }) {
	const { t } = useTranslation("admin");
	return (
		<button
			type="submit"
			disabled={disabled}
			className="justify-self-start border-[#5c3a21] border-2 bg-[#f4c95d] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
		>
			{disabled ? t("actions.saving") : t("actions.save")}
		</button>
	);
}
