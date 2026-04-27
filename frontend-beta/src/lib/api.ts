import type {
	UploadResponse,
	EmailAnalysisReport,
	HealthResponse,
	DashboardData,
} from "./types";

export class ApiError extends Error {
	constructor(
		public readonly status: number,
		message: string,
	) {
		super(message);
		this.name = "ApiError";
	}
}

const DEFAULT_API_BASE = "http://localhost:3001/api/v1";
const DEFAULT_WS_BASE = "ws://localhost:3001/api/v1";

function resolveApiBase(): string {
	if (process.env.NEXT_PUBLIC_API_URL) {
		return process.env.NEXT_PUBLIC_API_URL;
	}

	if (typeof window !== "undefined") {
		const protocol =
			window.location.protocol === "https:" ? "https" : "http";
		return `${protocol}://${window.location.hostname}:3001/api/v1`;
	}

	return DEFAULT_API_BASE;
}

function resolveWsBase(): string {
	if (process.env.NEXT_PUBLIC_WS_URL) {
		return process.env.NEXT_PUBLIC_WS_URL;
	}

	if (typeof window !== "undefined") {
		const protocol = window.location.protocol === "https:" ? "wss" : "ws";
		return `${protocol}://${window.location.hostname}:3001/api/v1`;
	}

	return DEFAULT_WS_BASE;
}

/** Get stored JWT token */
function getToken(): string | null {
	if (typeof window === "undefined") return null;
	return localStorage.getItem("deepmail_token");
}

export function getStoredToken(): string | null {
	return getToken();
}

/** Set JWT token */
export function setToken(token: string): void {
	localStorage.setItem("deepmail_token", token);
	// Trigger storage event manually for same-window reactivity
	window.dispatchEvent(new Event("storage_local_update"));
}

export function clearToken(): void {
	localStorage.removeItem("deepmail_token");
	// Trigger storage event manually for same-window reactivity
	window.dispatchEvent(new Event("storage_local_update"));
}

/** Build headers with auth */
function authHeaders(extra?: Record<string, string>): HeadersInit {
	const headers: Record<string, string> = {
		...extra,
	};
	const token = getToken();
	if (token) {
		headers["Authorization"] = `Bearer ${token}`;

		// Extract fingerprint from JWT cnf claim (required by backend)
		try {
			const payload = token.split(".")[1];
			// Support both browser atob and standard base64 decoding
			const decoded = JSON.parse(
				atob(payload.replace(/-/g, "+").replace(/_/g, "/")),
			);
			if (decoded.cnf) {
				headers["x-device-fingerprint"] = decoded.cnf;
			}
		} catch (error) {
			console.warn("Failed to extract fingerprint from JWT token", error);
		}
	}
	return headers;
}

/** Generic fetch wrapper with error handling */
async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const url = `${resolveApiBase()}${path}`;
	let res: Response;

	try {
		res = await fetch(url, {
			...init,
			headers: {
				...authHeaders(),
				...init?.headers,
			},
		});
	} catch (error) {
		throw new Error(
			`Unable to reach API at ${url}. Make sure the backend is running and reachable on your local/inhouse network.`,
			{ cause: error },
		);
	}

	if (!res.ok) {
		const body = await res.text().catch(() => "");
		throw new ApiError(
			res.status,
			`API ${res.status}: ${body || res.statusText}`,
		);
	}

	return res.json();
}

/** POST /api/v1/upload — Upload file for analysis */
export async function uploadFile(file: File): Promise<UploadResponse> {
	const formData = new FormData();
	formData.append("file", file);

	const url = `${resolveApiBase()}/upload`;
	const res = await fetch(url, {
		method: "POST",
		headers: authHeaders(),
		body: formData,
	});

	if (!res.ok) {
		const body = await res.text().catch(() => "");
		throw new ApiError(
			res.status,
			`Upload failed (${res.status}): ${body || res.statusText}`,
		);
	}

	return res.json();
}

/** GET /api/v1/results/:email_id — Fetch analysis report */
export async function getResults(
	emailId: string,
): Promise<EmailAnalysisReport> {
	return apiFetch<EmailAnalysisReport>(`/results/${emailId}`);
}

/** GET /api/v1/dashboard — Fetch dashboard data */
export async function getDashboard(): Promise<DashboardData> {
	return apiFetch<DashboardData>("/dashboard");
}

/** GET /api/v1/health — Health check */
export async function getHealth(): Promise<HealthResponse> {
	return apiFetch<HealthResponse>("/health");
}

/** Create WebSocket connection for live progress */
export function createProgressWebSocket(
	emailId: string,
	onMessage: (event: MessageEvent) => void,
	onError?: (event: Event) => void,
	onClose?: (event: CloseEvent) => void,
): WebSocket {
	const wsBase = resolveWsBase();
	const token = getToken();
	const url = `${wsBase}/ws/results/${emailId}${token ? `?token=${token}` : ""}`;

	const ws = new WebSocket(url);
	ws.onmessage = onMessage;
	if (onError) ws.onerror = onError;
	if (onClose) ws.onclose = onClose;

	return ws;
}
