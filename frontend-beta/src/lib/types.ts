// ── TypeScript interfaces matching Rust backend models ──────────────────────

/** Response from POST /api/v1/upload */
export interface UploadResponse {
	email_id: string;
	job_id: string;
	status: string;
	message: string;
	deduplicated: boolean;
}

/** Full analysis report from GET /api/v1/results/:email_id */
export interface EmailAnalysisReport {
	email: EmailSummary;
	analysis_results: AnalysisResultEntry[];
	job_progress: JobProgressEntry[];
	iocs: IocEntry[];
	geo_points?: GeoMapPoint[];
	hop_timeline?: HopTimelinePoint[];
}

export interface GeoMapPoint {
	id: string;
	ip: string;
	lat: number;
	lon: number;
	country: string;
	city?: string;
	region?: string;
	asn?: number;
	org?: string;
	risk: "critical" | "high" | "medium" | "low";
	abuse_confidence?: number;
	is_tor: boolean;
	is_proxy: boolean;
	confidence_score: number;
}

export interface HopTimelinePoint {
	hop: number;
	from_host?: string;
	by_host?: string;
	ip?: string;
}

/** Core email record */
export interface EmailSummary {
	id: string;
	original_name: string;
	sha256_hash: string;
	file_size: number;
	submitted_at: string;
	status: EmailStatus;
	current_stage?: string;
	completed_at?: string;
	error_message?: string;
}

export type EmailStatus =
	| "queued"
	| "processing"
	| "analyzing_headers"
	| "extracting_iocs"
	| "url_analysis"
	| "attachment_analysis"
	| "scoring"
	| "completed"
	| "failed";

/** Single analysis result entry (one per stage) */
export interface AnalysisResultEntry {
	id: string;
	result_type: string;
	data: Record<string, unknown>;
	threat_score?: number;
	confidence?: number;
	created_at: string;
}

export interface GeoPoint {
	id: string;
	lat: number;
	lon: number;
	country: string;
	risk: "Safe" | "Suspicious" | "Critical";
	value: string;
}

export interface DashboardStats {
	global_24h: number;
	malicious: number;
	suspicious: number;
	safe: number;
}

export interface TrendDataPoint {
	hour: string;
	safe: number;
	suspicious: number;
	malicious: number;
}

export interface RecentAnalysis {
	id: string;
	original_name: string;
	sender: string | null;
	status: string;
	submitted_at: string;
	risk_level: string;
	score: number;
}

export interface DashboardData {
	stats: DashboardStats;
	trend: TrendDataPoint[];
	recent_analyses: RecentAnalysis[];
	geo_points: GeoPoint[];
}

/** Pipeline stage timing record */
export interface JobProgressEntry {
	id: string;
	stage: string;
	status: string;
	started_at: string;
	completed_at?: string;
	details?: string;
}

/** IOC node linked to an email */
export interface IocEntry {
	id: string;
	ioc_type: IocType;
	value: string;
	first_seen: string;
	last_seen: string;
	metadata?: string;
}

export type IocType =
	| "ip"
	| "domain"
	| "url"
	| "md5"
	| "sha1"
	| "sha256"
	| "email";

/** Multi-dimensional threat score */
export interface ThreatScore {
	total: number;
	confidence: number;
	breakdown: ScoreBreakdown;
}

export interface ScoreBreakdown {
	identity: number;
	infrastructure: number;
	content: number;
	attachment: number;
}

/** Health check response */
export interface HealthResponse {
	status: string;
	database: boolean;
	redis: boolean;
	timestamp: string;
}

/** WebSocket progress event */
export interface ProgressEvent {
	email_id: string;
	stage: string;
	status: "started" | "completed" | "failed";
	details?: string;
}

/** Pipeline stage definition for UI */
export interface PipelineStage {
	key: string;
	label: string;
	icon: string;
	status: "pending" | "started" | "completed" | "failed";
	details?: string;
	timestamp?: string;
}

export const PIPELINE_STAGES: Omit<PipelineStage, "status">[] = [
	{ key: "parse_email", label: "Parse Email", icon: "drafts" },
	{ key: "header_analysis", label: "Header Analysis", icon: "policy" },
	{ key: "ioc_extraction", label: "IOC Extraction", icon: "search" },
	{ key: "phishing_keywords", label: "Phishing Scan", icon: "phishing" },
	{ key: "url_analysis", label: "URL Analysis", icon: "link" },
	{
		key: "attachment_analysis",
		label: "Attachment Analysis",
		icon: "attachment",
	},
	{ key: "threat_scoring", label: "Threat Scoring", icon: "speed" },
	{ key: "pipeline", label: "Complete", icon: "check_circle" },
];
