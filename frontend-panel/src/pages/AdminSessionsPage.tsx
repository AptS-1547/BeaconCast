import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { formatAdminTime } from "@/features/admin/adminModel";
import type { AdminSession } from "@/services/beaconService";
import { adminAuthService } from "@/services/beaconService";

export function AdminSessionsPage() {
	const { i18n, t } = useTranslation("admin");
	const [sessions, setSessions] = useState<AdminSession[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			setSessions(await adminAuthService.sessions());
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
			setSessions([]);
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
	}, [load]);

	async function handleRevoke(sessionId: number) {
		setLoading(true);
		setError(null);
		try {
			await adminAuthService.revokeSession(sessionId);
			await load();
		} catch (revokeError) {
			setError(
				revokeError instanceof Error ? revokeError.message : "revoke failed",
			);
			setLoading(false);
		}
	}

	return (
		<AdminPanel title={t("dashboard.sessions")}>
			{error ? (
				<p className="mt-3 border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}
			{sessions.length > 0 ? (
				<div className="mt-3 grid gap-2">
					{sessions.map((session) => (
						<div
							className="grid gap-2 border-[#c99c55] border-2 bg-[#f7e3a3] p-3 text-sm md:grid-cols-[80px_1fr_1fr_auto]"
							key={session.id}
						>
							<strong>#{session.id}</strong>
							<span>
								{t("session.expiredAt")}:{" "}
								{formatAdminTime(session.expires_at, i18n.language)}
							</span>
							<span>{session.current ? t("session.current") : ""}</span>
							<button
								type="button"
								disabled={
									loading || session.current || session.revoked_at !== null
								}
								onClick={() => void handleRevoke(session.id)}
								className="border-[#5c3a21] border-2 bg-[#f6b26b] px-2 py-1 font-black text-xs disabled:opacity-50"
							>
								{t("actions.revoke")}
							</button>
						</div>
					))}
				</div>
			) : (
				<p className="mt-3 text-[#6f543c]">{t("empty.sessions")}</p>
			)}
		</AdminPanel>
	);
}
