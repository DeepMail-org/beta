"use client";

import { useMemo, useState } from "react";
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
		<div className="flex flex-col gap-6 w-full max-w-7xl mx-auto">
			<div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
				<StatCard label="Open" value={String(counts.open)} change="+2 today" trend="up" />
				<StatCard label="Investigating" value={String(counts.inv)} change="-1" trend="down" />
				<StatCard label="Contained" value={String(counts.con)} />
				<StatCard label="Resolved (7d)" value={String(counts.res)} change="+5" trend="up" />
			</div>

			<Card
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

				<div className="overflow-x-auto w-full mt-4">
					<table className="w-full text-sm whitespace-nowrap">
						<thead className="border-b border-border text-muted text-xs">
							<tr>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Case ID</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Subject</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Owner</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Severity</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Status</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">SLA</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Updated</th>
							</tr>
						</thead>
						<tbody>
							{filtered.map((c, i) => (
								<tr key={c.id} className={`border-b border-white/[0.02] hover:bg-surface-2/50 transition-colors ${i % 2 ? "bg-white/[0.01]" : ""}`}>
									<td className="py-3 px-4 font-mono text-muted text-xs">{c.id}</td>
									<td className="py-3 px-4 max-w-[200px] truncate font-medium text-foreground">{c.subject}</td>
									<td className="py-3 px-4">
										<span className="inline-flex items-center justify-center w-6 h-6 rounded-full bg-surface-2 text-[10px] font-bold text-foreground">
											{c.owner}
										</span>
									</td>
									<td className="py-3 px-4"><Severity level={c.severity} /></td>
									<td className="py-3 px-4"><Tag tone={STATUS_TONES[c.status]}>{c.status.toUpperCase()}</Tag></td>
									<td className="py-3 px-4 font-mono text-[11px] text-muted">{c.sla}</td>
									<td className="py-3 px-4 text-muted text-xs">{c.updated}</td>
								</tr>
							))}
							{filtered.length === 0 && (
								<tr>
									<td colSpan={7} className="text-center py-8 text-muted">
										No cases match the current filter.
									</td>
								</tr>
							)}
						</tbody>
					</table>
				</div>

				<div className="mt-4 pt-4 border-t border-border flex items-center justify-between text-xs text-muted">
					<span>Showing {filtered.length} of {CASES.length}</span>
					<div className="flex items-center gap-2">
						<Btn size="sm">Prev</Btn>
						<Btn size="sm">Next</Btn>
					</div>
				</div>
			</Card>
		</div>
	);
}
