import { describe, expect, it } from "bun:test";
import {
	formatUtcDateTime,
	formatUtcTime,
	stableRotationFromId,
} from "./format";

describe("format helpers", () => {
	it("formats time deterministically in UTC", () => {
		expect(formatUtcTime("2026-03-29T21:58:47Z")).toBe("21:58:47");
	});

	it("formats datetime deterministically in UTC", () => {
		expect(formatUtcDateTime("2026-03-29T21:58:47Z")).toBe(
			"2026-03-29 21:58:47 UTC",
		);
	});

	it("generates stable rotation values per id", () => {
		const a = stableRotationFromId("ioc-1", 30);
		const b = stableRotationFromId("ioc-1", 30);
		const c = stableRotationFromId("ioc-2", 30);

		expect(a).toBe(b);
		expect(a).toBeGreaterThanOrEqual(-30);
		expect(a).toBeLessThanOrEqual(30);
		expect(c).not.toBe(a);
	});
});
