"use client";

import { useMemo, useState } from "react";
import AppShell from "@/components/dashboard/AppShell";
import { Btn, Card, Search, Severity, StatCard, Tabs, Tag } from "@/components/dashboard/primitives";

type CaseStatus = "open" | "investigating" | "contained" | "resolved";
type CaseSeverity = "critical" | "warning" | "caution" | "ok";

const CASES: { id: string; subject: string; owner: string; severity: CaseSeverity; status: CaseStatus; sla: string; updated: string }[] = [
	{ id: "CASE-2041", subject: "BEC attempt — finance team", owner: "VJ", severity: "critical", status: "investigating", sla: "1h 12m", updated: "2m ago" },
	{ id: "CASE-2040", subject: "Credential phishing — Apple ID lure", owner: "RS", severity: "critical", status: "open", sla: "2h 04m", updated: "9m ago" },
	{ id: "CASE-2039", subject: "Suspicious attachment .iso payload", owner: "AS", severity: "warning", status: "contained", sla: "4h 30m", updated: "21m ago" },
	{ id: "CASE-2038", subject: "Spoofed vendor invoice", owner: "VJ", severity: "warning", status: "investigating", sla: "5h 50m", updated: "44m ago" },
	{ id: "CASE-2037", subject: "Reconnaissance pattern detected", owner: "—", severity: "caution", status: "open", sla: "12h", updated: "1h ago" },
	{ id: "CASE-2036", subject: "Marketing newsletter false positive", owner: "RS", severity: "ok", status: "resolved", sla: "—", updated: "3h ago" },
	{ id: "CASE-2035", subject: "Lookalike domain — paypa1-secure.io", owner: "AS", severity: "warning", status: "contained", sla: "—", updated: "5h ago" },
];

const STATUS_TONES: Record<CaseStatus, "danger" | "warn" | "ok" | "default"> = {
	open: "danger",
	investigating: "warn",
	contained: "default",
	resolved: "ok",
};

export default function CasesPage() {
	const [tab, setTab] = useState<CaseStatus | "all">("all");
	const [q, setQ] = useState("");

	const filtered = useMemo(() => {
		return CASES.filter((c) => {
			if (tab !== "all" && c.status !== tab) return false;
			if (q && !`${c.id} ${c.subject} ${c.owner}`.toLowerCase().includes(q.toLowerCase())) return false;
			return true;
		});
	}, [tab, q]);

	const counts = useMemo(() => {
		const all = CASES.length;
		const open = CASES.filter((c) => c.status === "open").length;
		const inv = CASES.filter((c) => c.status === "investigating").length;
		const con = CASES.filter((c) => c.status === "contained").length;
		const res = CASES.filter((c) => c.status === "resolved").length;
		return { all, open, inv, con, res };
	}, []);

	return (
		<AppShell title="Investigation Cases" breadcrumb="DEEPMAIL · CASES">
			<div className="hf-stats-row">
				<StatCard label="Open" value={String(counts.open)} change="+2 today" trend="up" />
				<StatCard label="Investigating" value={String(counts.inv)} change="-1" trend="down" />
				<StatCard label="Contained" value={String(counts.con)} />
				<StatCard label="Resolved (7d)" value={String(counts.res)} change="+5" trend="up" />
			</div>

			<Card
				className="mt-6"
				title="Active Caseload"
				subtitle="Investigation queue and SLA tracking"
				actions={
					<>
						<Search placeholder="Search ID / subject / owner…" value={q} onChange={setQ} />
						<Btn size="sm" variant="accent">+ New Case</Btn>
					</>
				}
			>
				<Tabs
					tabs={[
						{ id: "all", label: "All", count: counts.all },
						{ id: "open", label: "Open", count: counts.open },
						{ id: "investigating", label: "Investigating", count: counts.inv },
						{ id: "contained", label: "Contained", count: counts.con },
						{ id: "resolved", label: "Resolved", count: counts.res },
					]}
					active={tab}
					onChange={(t) => setTab(t as CaseStatus | "all")}
				/>

				<div className="hf-table-wrap mt-4">
					<table className="w-full text-xs">
						<thead className="hf-table-header">
							<tr>
								<th className="text-left">Case ID</th>
								<th className="text-left">Subject</th>
								<th className="text-left">Owner</th>
								<th className="text-left">Severity</th>
								<th className="text-left">Status</th>
								<th className="text-left">SLA</th>
								<th className="text-left">Updated</th>
							</tr>
						</thead>
						<tbody>
							{filtered.map((c, i) => (
								<tr key={c.id} className={`hf-table-row ${i % 2 ? "hf-row-alt" : ""}`}>
									<td className="font-mono opacity-70">{c.id}</td>
									<td className="hf-cell-truncate font-medium">{c.subject}</td>
									<td>
										<span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-white/[0.06] text-[10px] font-bold">
											{c.owner}
										</span>
									</td>
									<td><Severity level={c.severity} /></td>
									<td><Tag tone={STATUS_TONES[c.status]}>{c.status.toUpperCase()}</Tag></td>
									<td className="font-mono text-[11px] opacity-70">{c.sla}</td>
									<td className="opacity-60">{c.updated}</td>
								</tr>
							))}
							{filtered.length === 0 && (
								<tr>
									<td colSpan={7} className="text-center py-8 opacity-50">
										No cases match the current filter.
									</td>
								</tr>
							)}
						</tbody>
					</table>
				</div>

				<div className="hf-table-footer">
					<span>Showing {filtered.length} of {CASES.length}</span>
					<div className="flex items-center gap-2">
						<Btn size="sm">Prev</Btn>
						<Btn size="sm">Next</Btn>
					</div>
				</div>
			</Card>
		</AppShell>
	);
}
