type JsonRequestInit = Omit<RequestInit, "headers"> & {
	headers?: HeadersInit;
	json?: unknown;
};

export class HttpError extends Error {
	readonly status: number;
	readonly payload: unknown;

	constructor(status: number, payload: unknown) {
		super(`HTTP request failed with status ${status}`);
		this.name = "HttpError";
		this.status = status;
		this.payload = payload;
	}
}

export async function requestJson<TResponse>(
	input: RequestInfo | URL,
	init: JsonRequestInit = {},
) {
	const headers = new Headers(init.headers);
	headers.set("Accept", "application/json");
	if (init.json !== undefined) {
		headers.set("Content-Type", "application/json");
	}
	const method = init.method?.toUpperCase() ?? "GET";
	if (isUnsafeMethod(method) && isAdminApi(input)) {
		const csrfToken = readCookie("beacon_admin_csrf");
		if (csrfToken) headers.set("X-Beacon-CSRF-Token", csrfToken);
	}

	const response = await fetch(input, {
		...init,
		body: init.json !== undefined ? JSON.stringify(init.json) : init.body,
		credentials: init.credentials ?? "include",
		headers,
	});
	const text = await response.text();
	const payload = text ? JSON.parse(text) : null;

	if (!response.ok) {
		throw new HttpError(response.status, payload);
	}

	return payload as TResponse;
}

type ApiEnvelope<T> = {
	code: string;
	msg: string;
	data?: T;
	error?: unknown;
};

function apiPath(path: string) {
	return path.startsWith("/api/v1") ? path : `/api/v1${path}`;
}

function isUnsafeMethod(method: string) {
	return !["GET", "HEAD", "OPTIONS", "TRACE"].includes(method);
}

function isAdminApi(input: RequestInfo | URL) {
	const value = String(input);
	return value.startsWith("/api/v1/admin") || value.startsWith("/admin");
}

function readCookie(name: string) {
	if (typeof document === "undefined") return null;
	const prefix = `${encodeURIComponent(name)}=`;
	return (
		document.cookie
			.split(";")
			.map((part) => part.trim())
			.find((part) => part.startsWith(prefix))
			?.slice(prefix.length) ?? null
	);
}

async function requestApi<TResponse>(
	path: string,
	init: JsonRequestInit = {},
): Promise<TResponse> {
	const envelope = await requestJson<ApiEnvelope<TResponse>>(apiPath(path), {
		...init,
	});
	if (envelope.data === undefined) {
		throw new HttpError(500, envelope);
	}
	return envelope.data;
}

export const api = {
	get: <TResponse>(path: string, init?: JsonRequestInit) =>
		requestApi<TResponse>(path, { ...init, method: "GET" }),
	post: <TResponse>(path: string, json?: unknown, init?: JsonRequestInit) =>
		requestApi<TResponse>(path, { ...init, method: "POST", json }),
	put: <TResponse>(path: string, json?: unknown, init?: JsonRequestInit) =>
		requestApi<TResponse>(path, { ...init, method: "PUT", json }),
	delete: <TResponse>(path: string, init?: JsonRequestInit) =>
		requestApi<TResponse>(path, { ...init, method: "DELETE" }),
};
