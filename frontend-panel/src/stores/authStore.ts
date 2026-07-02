import { create } from "zustand";
import { type AdminUser, adminAuthService } from "@/services/beaconService";

type AuthState = {
	initialized: boolean | null;
	isAuthenticated: boolean;
	isChecking: boolean;
	user: AdminUser | null;
	checkSetup: () => Promise<void>;
	checkAuth: () => Promise<void>;
	setup: (
		username: string,
		password: string,
		displayName: string,
	) => Promise<void>;
	login: (username: string, password: string) => Promise<void>;
	logout: () => Promise<void>;
};

export const useAuthStore = create<AuthState>((set, get) => ({
	initialized: null,
	isAuthenticated: false,
	isChecking: false,
	user: null,
	async checkSetup() {
		const response = await adminAuthService.check();
		set({ initialized: response.initialized });
	},
	async checkAuth() {
		set({ isChecking: true });
		try {
			const user = await adminAuthService.me();
			set({ isAuthenticated: true, isChecking: false, user });
		} catch (error) {
			set({ isAuthenticated: false, isChecking: false, user: null });
			throw error;
		}
	},
	async setup(username, password, displayName) {
		const response = await adminAuthService.setup({
			username,
			password,
			display_name: displayName,
		});
		set({
			initialized: true,
			isAuthenticated: true,
			user: response.user,
		});
	},
	async login(username, password) {
		const response = await adminAuthService.login({ username, password });
		set({
			isAuthenticated: true,
			user: response.user,
		});
	},
	async logout() {
		try {
			if (get().isAuthenticated) await adminAuthService.logout();
		} finally {
			set({
				isAuthenticated: false,
				user: null,
			});
		}
	},
}));
