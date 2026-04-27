import { z } from "zod";

export const GeoMapPointSchema = z.object({
	id: z.string(),
	ip: z.string(),
	lat: z.number(),
	lon: z.number(),
	country: z.string(),
	city: z.string().optional(),
	region: z.string().optional(),
	asn: z.number().optional(),
	org: z.string().optional(),
	risk: z.enum(["critical", "high", "medium", "low"]),
	abuse_confidence: z.number().optional(),
	is_tor: z.boolean(),
	is_proxy: z.boolean(),
	confidence_score: z.number(),
});

export const HopTimelinePointSchema = z.object({
	hop: z.number(),
	from_host: z.string().optional(),
	by_host: z.string().optional(),
	ip: z.string().optional(),
});

export const EmailAnalysisReportSchema = z.object({
	email: z.object({
		id: z.string(),
		original_name: z.string(),
		sha256_hash: z.string(),
		file_size: z.number(),
		submitted_at: z.string(),
		status: z.string(),
		current_stage: z.string().nullable().optional(),
		completed_at: z.string().nullable().optional(),
		error_message: z.string().nullable().optional(),
	}),
	analysis_results: z.array(z.unknown()),
	job_progress: z.array(z.unknown()),
	iocs: z.array(z.unknown()),
	geo_points: z.array(GeoMapPointSchema).default([]),
	hop_timeline: z.array(HopTimelinePointSchema).default([]),
});

export type ContractEmailAnalysisReport = z.infer<typeof EmailAnalysisReportSchema>;

export function parseEmailAnalysisReportContract(payload: unknown): ContractEmailAnalysisReport {
	return EmailAnalysisReportSchema.parse(payload);
}
