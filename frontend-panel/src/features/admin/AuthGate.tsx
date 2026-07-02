import type { FormEvent } from "react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "@/stores/authStore";
import { AdminPanel } from "./AdminPanel";

type AuthGateProps = {
	children: React.ReactNode;
};

export function AuthGate({ children }: AuthGateProps) {
	const { t } = useTranslation("admin");
	const initialized = useAuthStore((state) => state.initialized);
	const isAuthenticated = useAuthStore((state) => state.isAuthenticated);
	const checkSetup = useAuthStore((state) => state.checkSetup);
	const checkAuth = useAuthStore((state) => state.checkAuth);
	const login = useAuthStore((state) => state.login);
	const setup = useAuthStore((state) => state.setup);
	const [username, setUsername] = useState("");
	const [password, setPassword] = useState("");
	const [displayName, setDisplayName] = useState("");
	const [submitting, setSubmitting] = useState(false);
	const [error, setError] = useState<string | null>(null);

	useEffect(() => {
		void checkSetup().then(() => checkAuth().catch(() => undefined));
	}, [checkAuth, checkSetup]);

	async function handleSubmit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		setSubmitting(true);
		setError(null);
		try {
			if (initialized) {
				await login(username, password);
			} else {
				await setup(username, password, displayName);
			}
		} catch (submitError) {
			setError(
				submitError instanceof Error ? submitError.message : "auth failed",
			);
		} finally {
			setSubmitting(false);
		}
	}

	if (isAuthenticated) return <>{children}</>;

	const isSetup = initialized === false;

	return (
		<main className="grid min-h-svh place-items-center bg-[#7fb069] px-4 py-8 text-[#3f2a1d]">
			<AdminPanel className="w-full max-w-md">
				<p className="font-black text-[#7a4f2b] text-xs uppercase">
					BeaconCast
				</p>
				<h1 className="mt-1 font-black text-3xl">
					{isSetup ? t("auth.setupTitle") : t("auth.loginTitle")}
				</h1>
				<p className="mt-3 text-[#6f543c] leading-7">{t("auth.subtitle")}</p>
				<p className="mt-3 border-[#d7b66c] border-2 bg-[#f7e3a3] px-3 py-2 font-bold text-sm">
					{isSetup ? t("setup.needed") : t("setup.ready")}
				</p>
				<form className="mt-5 grid gap-4" onSubmit={handleSubmit}>
					<label className="grid gap-2 font-bold">
						<span>{t("auth.username")}</span>
						<input
							className="h-11 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
							value={username}
							onChange={(event) => setUsername(event.target.value)}
							autoComplete="username"
							required
						/>
					</label>
					{isSetup ? (
						<label className="grid gap-2 font-bold">
							<span>{t("auth.displayName")}</span>
							<input
								className="h-11 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
								value={displayName}
								onChange={(event) => setDisplayName(event.target.value)}
								autoComplete="name"
								required
							/>
						</label>
					) : null}
					<label className="grid gap-2 font-bold">
						<span>{t("auth.password")}</span>
						<input
							className="h-11 border-[#5c3a21] border-2 bg-white px-3 outline-none focus:ring-4 focus:ring-[#8bbbd9]"
							type="password"
							value={password}
							onChange={(event) => setPassword(event.target.value)}
							autoComplete={isSetup ? "new-password" : "current-password"}
							required
						/>
					</label>
					{error ? (
						<p className="border-[#b5534c] border-2 bg-[#ffe1d8] px-3 py-2 font-bold text-[#7f2e28]">
							{error}
						</p>
					) : null}
					<button
						type="submit"
						disabled={submitting || initialized === null}
						className="border-[#5c3a21] border-2 bg-[#f4c95d] px-4 py-3 font-black shadow-[3px_3px_0_#5c3a21] transition hover:-translate-y-0.5 disabled:cursor-not-allowed disabled:opacity-60"
					>
						{isSetup ? t("actions.setup") : t("actions.login")}
					</button>
				</form>
			</AdminPanel>
		</main>
	);
}
