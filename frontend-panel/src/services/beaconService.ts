import { api } from "@/services/httpClient";
import type { components } from "@/types/api.generated";

export type ActivityLogEntry = components["schemas"]["ActivityLogEntry"];
export type ActivityKind = string;
export type ActivityStatus = string;
export type ActivitySummary = components["schemas"]["ActivitySummaryResponse"];
export type ActivityAction = components["schemas"]["ActivityActionResponse"];
export type ActivityApplication =
	components["schemas"]["ActivityApplicationResponse"];
export type AdminActivityEvent =
	components["schemas"]["AdminActivityEventResponse"];
export type AdminUsageSpan = components["schemas"]["AdminUsageSpanResponse"];
export type AdminUsageSummary =
	components["schemas"]["AdminUsageSummaryResponse"];
export type AdminAuthCheck = components["schemas"]["AdminAuthCheckResponse"];
export type AdminAuthResponse = components["schemas"]["AdminAuthResponse"];
export type AdminLoginRequest = components["schemas"]["AdminLoginRequest"];
export type AdminSession = components["schemas"]["AdminSessionResponse"];
export type AdminSetupRequest = components["schemas"]["AdminSetupRequest"];
export type AdminUser = components["schemas"]["AdminUserResponse"];
export type AgentPolicy = components["schemas"]["AgentPolicyResponse"];
export type BeaconDevice = components["schemas"]["BeaconDeviceResponse"];
export type BeaconDeviceToken =
	components["schemas"]["BeaconDeviceTokenResponse"];
export type CreateBeaconDeviceRequest =
	components["schemas"]["CreateBeaconDeviceRequest"];
export type CreateBeaconDeviceTokenRequest =
	components["schemas"]["CreateBeaconDeviceTokenAdminRequest"];
export type CreateBeaconDeviceTokenResponse =
	components["schemas"]["CreateBeaconDeviceTokenAdminResponse"];
export type Cursor = components["schemas"]["DateTimeIdCursor"];
export type ActivityLogPage =
	components["schemas"]["CursorPage_ActivityLogEntry_DateTimeIdCursor"];
export type AdminActivityEventPage =
	components["schemas"]["CursorPage_AdminActivityEventResponse_DateTimeIdCursor"];
export type AdminUsageSpanPage =
	components["schemas"]["CursorPage_AdminUsageSpanResponse_DateTimeIdCursor"];
export type ManualOverride = components["schemas"]["ManualOverrideResponse"];
export type NowResponse = components["schemas"]["NowResponse"];
export type PublicActivity = components["schemas"]["PublicActivity"];
export type PublicMessagePart = components["schemas"]["PublicMessagePart"];
export type SetManualOverrideRequest =
	components["schemas"]["SetManualOverrideRequest"];
export type UpsertActivityActionRequest =
	components["schemas"]["UpsertActivityActionRequest"];
export type UpsertActivityApplicationRequest =
	components["schemas"]["UpsertActivityApplicationRequest"];
export type UpdateAgentPolicyRequest =
	components["schemas"]["UpdateAgentPolicyRequest"];
export type VisibilityPolicy =
	components["schemas"]["VisibilityPolicyResponse"];
export type UpdateVisibilityPolicyRequest =
	components["schemas"]["UpdateVisibilityPolicyRequest"];

function withQuery(path: string, params: Record<string, unknown>) {
	const query = new URLSearchParams();
	for (const [key, value] of Object.entries(params)) {
		if (value !== undefined && value !== null && value !== "") {
			query.set(key, String(value));
		}
	}
	const value = query.toString();
	return value ? `${path}?${value}` : path;
}

export const publicBeaconService = {
	now: () => api.get<NowResponse>("/beacon/now"),
	activityLog: (limit = 12, cursor?: Cursor | null) =>
		api.get<ActivityLogPage>(
			withQuery("/beacon/activity-log", {
				limit,
				after_created_at: cursor?.value,
				after_id: cursor?.id,
			}),
		),
	summary: () => api.get<ActivitySummary>("/beacon/activity-summary"),
};

export const adminAuthService = {
	check: () => api.get<AdminAuthCheck>("/admin/auth/check"),
	setup: (data: AdminSetupRequest) =>
		api.post<AdminAuthResponse>("/admin/auth/setup", data),
	login: (data: AdminLoginRequest) =>
		api.post<AdminAuthResponse>("/admin/auth/login", data),
	logout: () => api.post<{ revoked: boolean }>("/admin/auth/logout"),
	me: () => api.get<AdminUser>("/admin/me"),
	sessions: () => api.get<AdminSession[]>("/admin/sessions"),
	revokeSession: (sessionId: number) =>
		api.post<{ revoked: boolean }>(`/admin/sessions/${sessionId}/revoke`),
};

export const adminBeaconService = {
	devices: () => api.get<BeaconDevice[]>("/admin/beacon-devices"),
	createDevice: (data: CreateBeaconDeviceRequest) =>
		api.post<BeaconDevice>("/admin/beacon-devices", data),
	createDeviceToken: (deviceId: number, data: CreateBeaconDeviceTokenRequest) =>
		api.post<CreateBeaconDeviceTokenResponse>(
			`/admin/beacon-devices/${deviceId}/tokens`,
			data,
		),
	deviceTokens: (deviceId: number) =>
		api.get<BeaconDeviceToken[]>(`/admin/beacon-devices/${deviceId}/tokens`),
	disableDevice: (deviceId: number) =>
		api.post<{ changed: boolean }>(`/admin/beacon-devices/${deviceId}/disable`),
	enableDevice: (deviceId: number) =>
		api.post<{ changed: boolean }>(`/admin/beacon-devices/${deviceId}/enable`),
	revokeDeviceToken: (deviceId: number, tokenId: number) =>
		api.post<{ revoked: boolean }>(
			`/admin/beacon-devices/${deviceId}/tokens/${tokenId}/revoke`,
		),
	events: (limit = 20, cursor?: Cursor | null) =>
		api.get<AdminActivityEventPage>(
			withQuery("/admin/events", {
				limit,
				after_created_at: cursor?.value,
				after_id: cursor?.id,
			}),
		),
	usageSpans: (limit = 20, cursor?: Cursor | null) =>
		api.get<AdminUsageSpanPage>(
			withQuery("/admin/usage-spans", {
				limit,
				after_created_at: cursor?.value,
				after_id: cursor?.id,
			}),
		),
	usageSummary: (days = 1) =>
		api.get<AdminUsageSummary>(
			withQuery("/admin/usage-summary", {
				days,
			}),
		),
	activityActions: () => api.get<ActivityAction[]>("/admin/activity-actions"),
	upsertActivityAction: (
		actionKey: string,
		data: UpsertActivityActionRequest,
	) => api.put<ActivityAction>(`/admin/activity-actions/${actionKey}`, data),
	activityApplications: () =>
		api.get<ActivityApplication[]>("/admin/activity-applications"),
	upsertActivityApplication: (
		appKey: string,
		data: UpsertActivityApplicationRequest,
	) =>
		api.put<ActivityApplication>(
			`/admin/activity-applications/${appKey}`,
			data,
		),
	visibilityPolicy: () => api.get<VisibilityPolicy>("/admin/visibility-policy"),
	updateVisibilityPolicy: (data: UpdateVisibilityPolicyRequest) =>
		api.put<VisibilityPolicy>("/admin/visibility-policy", data),
	agentPolicy: () => api.get<AgentPolicy>("/admin/agent-policy"),
	updateAgentPolicy: (data: UpdateAgentPolicyRequest) =>
		api.put<AgentPolicy>("/admin/agent-policy", data),
	manualOverride: () => api.get<ManualOverride>("/admin/manual-override"),
	setManualOverride: (data: SetManualOverrideRequest) =>
		api.post<ManualOverride>("/admin/manual-override", data),
	clearManualOverride: () =>
		api.delete<{ cleared: boolean }>("/admin/manual-override"),
};
