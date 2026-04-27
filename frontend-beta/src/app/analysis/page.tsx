"use client";

import { useState } from "react";
import { formatUtcDateTime } from "@/lib/format";

const DUMMY_ANALYSES = [
	{
		id: "demoid1",
		original_name: "invoice_urgent.eml",
		risk_level: "Critical",
		score: 92,
		submitted_at: "2026-03-29T21:56:47Z",
		sender: "attacker@evil.com",
		status: "Analyzed",
	},
	{
		id: "demoid2",
		original_name: "marketing_update.msg",
		risk_level: "Suspicious",
		score: 55,
		submitted_at: "2026-03-29T21:52:47Z",
		sender: "newsletter@promo.com",
		status: "Analyzed",
	},
	{
		id: "demoid3",
		original_name: "meeting_notes.pdf",
		risk_level: "Safe",
		score: 10,
		submitted_at: "2026-03-29T21:50:47Z",
		sender: "colleague@company.com",
		status: "Analyzed",
	},
	{
		id: "demoid4",
		original_name: "password_reset.eml",
		risk_level: "Critical",
		score: 95,
		submitted_at: "2026-03-29T21:48:47Z",
		sender: "admin@it-support-fake.com",
		status: "Analyzed",
	},
	{
		id: "demoid5",
		original_name: "project_timeline.xlsx",
		risk_level: "Suspicious",
		score: 45,
		submitted_at: "2026-03-29T21:38:47Z",
		sender: "contractor@external.com",
		status: "Analyzed",
	},
	{
		id: "demoid6",
		original_name: "system_update.zip",
		risk_level: "Critical",
		score: 99,
		submitted_at: "2026-03-29T20:58:47Z",
		sender: "noreply@sysadmin.com",
		status: "Analyzed",
	},
];

export default function AnalysisIndexPage() {
	const [searchQuery, setSearchQuery] = useState("");

	const filteredAnalyses = DUMMY_ANALYSES.filter(
		(ra) =>
			ra.original_name
				.toLowerCase()
				.includes(searchQuery.toLowerCase()) ||
			ra.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
			ra.sender.toLowerCase().includes(searchQuery.toLowerCase()),
	);

	return (
		<div className="p-8 lg:p-12 space-y-8 max-w-7xl mx-auto">
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-4xl font-bold text-on-surface tracking-tight font-headline">
						ANALYSIS_RECORDS
					</h2>
					<p className="text-on-surface-variant mt-2 text-sm">
						Historical archive of all processed payloads and emails.
					</p>
				</div>
				<div className="relative w-64">
					<span className="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-outline text-sm">
						search
					</span>
					<input
						type="text"
						placeholder="Search Hash or Name..."
						value={searchQuery}
						onChange={(e) => setSearchQuery(e.target.value)}
						className="w-full bg-surface-container-low border border-outline-variant/20 rounded-lg pl-9 pr-4 py-2 text-sm text-on-surface focus:outline-none focus:border-primary/50 transition-colors"
					/>
				</div>
			</div>

			<div className="glass-panel rounded-xl overflow-hidden">
				<table className="w-full text-left">
					<thead className="text-[10px] uppercase tracking-widest text-outline bg-bg/40 border-b border-outline-variant/10">
						<tr>
							<th className="px-8 py-4 font-bold">
								Filename / Hash
							</th>
							<th className="px-8 py-4 font-bold">
								Sender Source
							</th>
							<th className="px-8 py-4 font-bold">Timestamp</th>
							<th className="px-8 py-4 font-bold">
								Security Grade
							</th>
							<th className="px-8 py-4 font-bold text-right">
								Actions
							</th>
						</tr>
					</thead>
					<tbody className="text-sm">
						{filteredAnalyses.length === 0 ? (
							<tr>
								<td
									colSpan={5}
									className="text-center py-12 text-outline text-xs uppercase tracking-widest"
								>
									No records found matching "{searchQuery}"
								</td>
							</tr>
						) : (
							filteredAnalyses.map((ra) => {
								let dotColor = "bg-tertiary";
								let badgeColor =
									"text-tertiary bg-tertiary/10 border-tertiary/20";
								if (ra.risk_level === "Critical") {
									dotColor = "bg-dracula-red";
									badgeColor =
										"text-dracula-red bg-dracula-red/10 border-dracula-red/20";
								} else if (ra.risk_level === "Suspicious") {
									dotColor = "bg-dracula-orange";
									badgeColor =
										"text-dracula-orange bg-dracula-orange/10 border-dracula-orange/20";
								}

								return (
									<tr
										key={ra.id}
										className="hover:bg-surface-container-high/40 transition-colors border-b border-outline-variant/5"
									>
										<td className="px-8 py-4">
											<div className="flex items-center gap-3">
												<div
													className={`w-2 h-2 rounded-full ${dotColor}`}
												/>
												<div>
													<p className="font-bold text-on-surface tracking-tight">
														{ra.original_name}
													</p>
													<p className="text-[10px] text-outline font-mono mt-0.5">
														ID: {ra.id}
													</p>
												</div>
											</div>
										</td>
										<td className="px-8 py-4 text-outline font-mono text-xs">
											{ra.sender}
										</td>
										<td className="px-8 py-4 text-outline font-mono text-xs">
											{formatUtcDateTime(ra.submitted_at)}
										</td>
										<td className="px-8 py-4">
											<div className="flex items-center gap-2">
												<span
													className={`text-[10px] px-2 py-1 rounded border font-bold uppercase ${badgeColor}`}
												>
													{ra.risk_level}
												</span>
												<span
													className={`font-bold text-xs ${badgeColor.split(" ")[0]}`}
												>
													{ra.score}/100
												</span>
											</div>
										</td>
										<td className="px-8 py-4 text-right">
											<a
												href={`/analysis/${ra.id}`}
												className="inline-block px-4 py-1.5 bg-surface-container border border-outline-variant/20 hover:border-outline/50 text-[10px] uppercase font-bold tracking-widest text-on-surface rounded transition-colors"
											>
												View Report
											</a>
										</td>
									</tr>
								);
							})
						)}
					</tbody>
				</table>
			</div>
		</div>
	);
}
