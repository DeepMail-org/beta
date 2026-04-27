import { describe, expect, test } from "bun:test";
import { parseEmailAnalysisReportContract } from "@/lib/contracts/results";

describe("results payload contract", () => {
	test("accepts geo_points and hop_timeline payload", () => {
		const payload = {
			email: {
				id: "e1",
				original_name: "mail.eml",
				sha256_hash: "abc",
				file_size: 100,
				submitted_at: "2026-01-01T00:00:00Z",
				status: "completed",
			},
			analysis_results: [],
			job_progress: [],
			iocs: [],
			geo_points: [
				{
					id: "g1",
					ip: "8.8.8.8",
					lat: 10,
					lon: 20,
					country: "US",
					risk: "medium",
					is_tor: false,
					is_proxy: false,
					confidence_score: 0.8,
				},
			],
			hop_timeline: [
				{ hop: 1, from_host: "a.example", by_host: "b.example", ip: "8.8.8.8" },
			],
		};

		const parsed = parseEmailAnalysisReportContract(payload);
		expect(parsed.geo_points.length).toBe(1);
		expect(parsed.hop_timeline.length).toBe(1);
	});

	test("rejects invalid risk enum", () => {
		const payload = {
			email: {
				id: "e1",
				original_name: "mail.eml",
				sha256_hash: "abc",
				file_size: 100,
				submitted_at: "2026-01-01T00:00:00Z",
				status: "completed",
			},
			analysis_results: [],
			job_progress: [],
			iocs: [],
			geo_points: [
				{
					id: "g1",
					ip: "8.8.8.8",
					lat: 10,
					lon: 20,
					country: "US",
					risk: "unknown",
					is_tor: false,
					is_proxy: false,
					confidence_score: 0.8,
				},
			],
			hop_timeline: [],
		};

		expect(() => parseEmailAnalysisReportContract(payload)).toThrow();
	});
});
