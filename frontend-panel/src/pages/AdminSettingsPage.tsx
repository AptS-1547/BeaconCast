import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { PolicyPanel } from "@/features/admin/PolicyPanel";
import type {
	AgentPolicy,
	ManualOverride,
	VisibilityPolicy,
} from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";

export function AdminSettingsPage() {
	const { t } = useTranslation("admin");
	const [agentPolicy, setAgentPolicy] = useState<AgentPolicy | null>(null);
	const [visibilityPolicy, setVisibilityPolicy] =
		useState<VisibilityPolicy | null>(null);
	const [manualOverride, setManualOverride] = useState<ManualOverride | null>(
		null,
	);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			const [visibility, agent, overrideData] = await Promise.all([
				adminBeaconService.visibilityPolicy(),
				adminBeaconService.agentPolicy(),
				adminBeaconService.manualOverride(),
			]);
			setVisibilityPolicy(visibility);
			setAgentPolicy(agent);
			setManualOverride(overrideData);
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
	}, [load]);

	return (
		<>
			{error ? (
				<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}
			<PolicyPanel
				agentPolicy={agentPolicy}
				loading={loading}
				visibilityPolicy={visibilityPolicy}
				onSaved={load}
			/>
			<AdminPanel title={t("dashboard.manualOverride")}>
				<p className="mt-3 font-bold text-[#6f543c]">
					{manualOverride?.active ? manualOverride.activity : "-"}
				</p>
			</AdminPanel>
		</>
	);
}
