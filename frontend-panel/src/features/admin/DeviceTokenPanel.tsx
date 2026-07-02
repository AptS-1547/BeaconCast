import type { FormEvent } from "react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type {
	BeaconDevice,
	BeaconDeviceToken,
	CreateBeaconDeviceTokenResponse,
} from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";
import { formatAdminTime } from "./adminModel";

type DeviceTokenPanelProps = {
	device: BeaconDevice;
	language: string;
};

export function DeviceTokenPanel({ device, language }: DeviceTokenPanelProps) {
	const { t } = useTranslation("admin");
	const [tokens, setTokens] = useState<BeaconDeviceToken[]>([]);
	const [createdToken, setCreatedToken] =
		useState<CreateBeaconDeviceTokenResponse | null>(null);
	const [tokenName, setTokenName] = useState("local-agent");
	const [loading, setLoading] = useState(false);
	const [error, setError] = useState<string | null>(null);

	const loadTokens = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			setTokens(await adminBeaconService.deviceTokens(device.id));
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
		} finally {
			setLoading(false);
		}
	}, [device.id]);

	useEffect(() => {
		void loadTokens();
	}, [loadTokens]);

	async function handleCreateToken(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setLoading(true);
		setError(null);
		try {
			const response = await adminBeaconService.createDeviceToken(device.id, {
				name: tokenName,
			});
			setCreatedToken(response);
			setTokenName("local-agent");
			await loadTokens();
		} catch (createError) {
			setError(
				createError instanceof Error ? createError.message : "create failed",
			);
			setLoading(false);
		}
	}

	async function handleRevoke(tokenId: number) {
		setLoading(true);
		setError(null);
		try {
			await adminBeaconService.revokeDeviceToken(device.id, tokenId);
			await loadTokens();
		} catch (revokeError) {
			setError(
				revokeError instanceof Error ? revokeError.message : "revoke failed",
			);
			setLoading(false);
		}
	}

	return (
		<div className="mt-3 grid gap-3 border-[#c99c55] border-2 bg-[#fff8db] p-3">
			<form
				className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto]"
				onSubmit={handleCreateToken}
			>
				<label className="grid gap-1 font-bold text-sm">
					<span>{t("dashboard.tokens")}</span>
					<input
						className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
						value={tokenName}
						onChange={(event) => setTokenName(event.target.value)}
						required
					/>
				</label>
				<button
					type="submit"
					disabled={loading}
					className="self-end border-[#5c3a21] border-2 bg-[#f4c95d] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
				>
					{t("actions.createToken")}
				</button>
			</form>

			{createdToken ? (
				<div className="grid gap-2 border-[#5c3a21] border-2 bg-[#d4ecaf] p-3">
					<p className="font-black">{t("tokens.createdToken")}</p>
					<code className="block overflow-x-auto border-[#7a4f2b] border-2 bg-white px-3 py-2 font-mono text-sm">
						{createdToken.token}
					</code>
				</div>
			) : null}

			{error ? (
				<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}

			{tokens.length > 0 ? (
				<div className="grid gap-2">
					{tokens.map((token) => (
						<div
							className="grid gap-2 border-[#e0bd6a] border-2 bg-[#f7e3a3] p-2 text-sm lg:grid-cols-[minmax(120px,1fr)_minmax(110px,auto)_minmax(110px,auto)_auto]"
							key={token.id}
						>
							<strong>{token.name}</strong>
							<span>
								{t("fields.lastUsedAt")}:{" "}
								{formatAdminTime(token.last_used_at, language)}
							</span>
							<span>
								{t("fields.revokedAt")}:{" "}
								{formatAdminTime(token.revoked_at, language)}
							</span>
							<button
								type="button"
								disabled={loading || token.revoked_at !== null}
								onClick={() => void handleRevoke(token.id)}
								className="border-[#5c3a21] border-2 bg-[#f6b26b] px-2 py-1 font-black text-xs disabled:opacity-50"
							>
								{t("actions.revoke")}
							</button>
						</div>
					))}
				</div>
			) : (
				<p className="text-[#6f543c]">{t("empty.tokens")}</p>
			)}
		</div>
	);
}
