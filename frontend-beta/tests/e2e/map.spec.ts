import { expect, test } from "@playwright/test";

const baseReport = {
	email: {
		id: "demo-1",
		original_name: "sample.eml",
		sha256_hash: "abc",
		file_size: 100,
		submitted_at: "2026-01-01T00:00:00Z",
		status: "completed",
		current_stage: null,
		completed_at: "2026-01-01T00:00:01Z",
		error_message: null,
	},
	analysis_results: [],
	job_progress: [],
	iocs: [{ id: "ip-1", ioc_type: "ip", value: "8.8.8.8", first_seen: "x", last_seen: "x", metadata: null }],
	geo_points: [
		{
			id: "ip-1",
			ip: "8.8.8.8",
			lat: 37.386,
			lon: -122.0838,
			country: "United States",
			city: "Mountain View",
			region: "California",
			asn: 15169,
			org: "Google LLC",
			risk: "medium",
			abuse_confidence: 10,
			is_tor: false,
			is_proxy: false,
			confidence_score: 0.92,
		},
	],
	hop_timeline: [
		{ hop: 3, from_host: "sender.example", by_host: "mx-a", ip: "8.8.8.8" },
	],
};

test("clicking map node opens enriched sidebar", async ({ page }) => {
	await page.route("**/api/v1/results/**", async (route) => {
		await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(baseReport) });
	});

	await page.goto("/analysis/demo-1/map");
	await expect(page.getByText("Global IP Geolocation")).toBeVisible();

	await page.evaluate(() => {
		const markers = Array.from(
			document.querySelectorAll<SVGPathElement>("path.leaflet-interactive"),
		);
		const visible = markers.find((el) => {
			const box = el.getBoundingClientRect();
			return box.width > 0 && box.height > 0;
		});
		(visible ?? markers[0])?.dispatchEvent(
			new MouseEvent("click", { bubbles: true, cancelable: true }),
		);
	});

	await expect(page.getByText("Selected Node")).toBeVisible();
	await expect(page.getByRole("link", { name: /View Full Report/i })).toBeVisible();
});

test("cluster behavior renders without crash for >100 points", async ({ page }) => {
	const manyPoints = Array.from({ length: 140 }, (_, index) => ({
		id: `ip-${index}`,
		ip: `11.0.0.${index % 255}`,
		lat: 20 + (index % 8) * 0.05,
		lon: 77 + (index % 8) * 0.05,
		country: "India",
		city: "Mumbai",
		region: "Maharashtra",
		asn: 64512,
		org: "Example Org",
		risk: "high",
		abuse_confidence: 30,
		is_tor: false,
		is_proxy: false,
		confidence_score: 0.77,
	}));

	await page.route("**/api/v1/results/**", async (route) => {
		await route.fulfill({
			status: 200,
			contentType: "application/json",
			body: JSON.stringify({ ...baseReport, geo_points: manyPoints }),
		});
	});

	await page.goto("/analysis/demo-1/map");
	await expect(page.getByText("Global IP Geolocation")).toBeVisible();
	await expect(page.locator(".leaflet-container")).toBeVisible();
});

test("retry flow works after API error", async ({ page }) => {
	let calls = 0;
	await page.route("**/api/v1/results/**", async (route) => {
		calls += 1;
		if (calls === 1) {
			await route.fulfill({ status: 500, contentType: "application/json", body: "{}" });
			return;
		}
		await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(baseReport) });
	});

	await page.goto("/analysis/demo-1/map");
	await expect(page.getByText("Map loading failed")).toBeVisible();
	await page.getByRole("button", { name: /Retry/i }).click();
	await expect(page.getByText("Global IP Geolocation")).toBeVisible();
});
