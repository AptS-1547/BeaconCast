import type { FormEvent } from "react";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AdminPanel } from "@/features/admin/AdminPanel";
import { DeviceTokenPanel } from "@/features/admin/DeviceTokenPanel";
import type { BeaconDevice } from "@/services/beaconService";
import { adminBeaconService } from "@/services/beaconService";

export function AdminDevicesPage() {
	const { i18n, t } = useTranslation("admin");
	const [devices, setDevices] = useState<BeaconDevice[]>([]);
	const [loading, setLoading] = useState(true);
	const [error, setError] = useState<string | null>(null);
	const [deviceKey, setDeviceKey] = useState("");
	const [deviceName, setDeviceName] = useState("");
	const [deviceKind, setDeviceKind] = useState("desktop");
	const [devicePriority, setDevicePriority] = useState(0);

	const load = useCallback(async () => {
		setLoading(true);
		setError(null);
		try {
			setDevices(await adminBeaconService.devices());
		} catch (loadError) {
			setError(loadError instanceof Error ? loadError.message : "load failed");
		} finally {
			setLoading(false);
		}
	}, []);

	useEffect(() => {
		void load();
	}, [load]);

	async function handleCreateDevice(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setLoading(true);
		setError(null);
		try {
			await adminBeaconService.createDevice({
				device_key: deviceKey,
				display_name: deviceName,
				kind: deviceKind,
				priority: devicePriority,
			});
			setDeviceKey("");
			setDeviceName("");
			setDeviceKind("desktop");
			setDevicePriority(0);
			await load();
		} catch (createError) {
			setError(
				createError instanceof Error ? createError.message : "create failed",
			);
			setLoading(false);
		}
	}

	return (
		<AdminPanel title={t("dashboard.devices")}>
			{error ? (
				<p className="mt-3 border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
					{error}
				</p>
			) : null}
			<form
				className="mt-3 grid gap-3 border-[#c99c55] border-2 bg-[#f7e3a3] p-3 lg:grid-cols-[minmax(120px,1fr)_minmax(140px,1fr)_120px_100px_auto]"
				onSubmit={handleCreateDevice}
			>
				<TextInput
					label={t("fields.deviceKey")}
					onChange={setDeviceKey}
					placeholder={t("tokens.deviceKeyPlaceholder")}
					value={deviceKey}
				/>
				<TextInput
					label={t("fields.displayName")}
					onChange={setDeviceName}
					placeholder={t("tokens.displayNamePlaceholder")}
					value={deviceName}
				/>
				<TextInput
					label={t("fields.kind")}
					onChange={setDeviceKind}
					value={deviceKind}
				/>
				<label className="grid gap-1 font-bold text-sm">
					<span>{t("fields.priority")}</span>
					<input
						className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
						type="number"
						value={devicePriority}
						onChange={(event) => setDevicePriority(Number(event.target.value))}
					/>
				</label>
				<button
					type="submit"
					disabled={loading}
					className="self-end border-[#5c3a21] border-2 bg-[#f4c95d] px-3 py-2 font-black text-sm shadow-[3px_3px_0_#5c3a21] disabled:opacity-60"
				>
					{t("actions.createDevice")}
				</button>
			</form>

			{devices.length > 0 ? (
				<div className="mt-3 grid gap-3">
					{devices.map((device) => (
						<article
							className="border-[#c99c55] border-2 bg-[#f7e3a3] p-3"
							key={device.id}
						>
							<div className="flex items-start justify-between gap-3">
								<strong>{device.display_name}</strong>
								<span className="font-black text-xs">#{device.priority}</span>
							</div>
							<p className="mt-2 text-[#6f543c] text-sm">{device.device_key}</p>
							<p className="mt-1 text-[#6f543c] text-sm">
								{t("fields.kind")}: {device.kind}
							</p>
							{device.capabilities.length > 0 ? (
								<div className="mt-2 flex flex-wrap gap-2">
									{device.capabilities.map((capability) => (
										<span
											className="border-[#5c3a21] border-2 bg-[#d9ead3] px-2 py-1 font-black text-xs"
											key={capability}
										>
											{t(`capabilities.${capability}`)}
										</span>
									))}
								</div>
							) : null}
							<DeviceTokenPanel device={device} language={i18n.language} />
						</article>
					))}
				</div>
			) : (
				<p className="mt-3 text-[#6f543c]">{t("empty.devices")}</p>
			)}
		</AdminPanel>
	);
}

function TextInput({
	label,
	onChange,
	placeholder,
	value,
}: {
	label: string;
	onChange: (value: string) => void;
	placeholder?: string;
	value: string;
}) {
	return (
		<label className="grid gap-1 font-bold text-sm">
			<span>{label}</span>
			<input
				className="h-10 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
				value={value}
				placeholder={placeholder}
				onChange={(event) => onChange(event.target.value)}
				required
			/>
		</label>
	);
}
