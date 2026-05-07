"use client";

import { useMemo, useState } from "react";
import { Btn, Card, Search, Severity, Tabs, Tag } from "@/components/dashboard/primitives";

type Action = "delivered" | "quarantined" | "blocked" | "flagged";
type Sev = "critical" | "warning" | "caution" | "ok";

const LOGS: { id: string; time: string; sender: string; recipient: string; subject: string; score: number; sev: Sev; action: Action }[] = [
	{ id: "EM-9821", time: "14:08:22", sender: "cfo@acme-corp.tk", recipient: "billing@deepmail.io", subject: "URGENT: Wire transfer authorization", score: 96, sev: "critical", action: "blocked" },
	{ id: "EM-9820", time: "14:07:14", sender: "no-reply@apple-id.support", recipient: "vyom@deepmail.io", subject: "Your Apple ID password was reset", score: 91, sev: "critical", action: "quarantined" },
	{ id: "EM-9819", time: "14:05:41", sender: "billing@vendor-portal.app", recipient: "ops@deepmail.io", subject: "Invoice #1224 — overdue notice", score: 78, sev: "warning", action: "flagged" },
	{ id: "EM-9818", time: "14:02:09", sender: "alerts@dropbox-share.app", recipient: "hr@deepmail.io", subject: "Shared file: Q3-Forecast.xlsx", score: 71, sev: "warning", action: "quarantined" },
	{ id: "EM-9817", time: "13:58:50", sender: "newsletter@promo.com", recipient: "marketing@deepmail.io", subject: "Weekly product highlights", score: 18, sev: "ok", action: "delivered" },
	{ id: "EM-9816", time: "13:54:31", sender: "ops@known-vendor.com", recipient: "ops@deepmail.io", subject: "Re: Vendor onboarding documents", score: 54, sev: "caution", action: "flagged" },
	{ id: "EM-9815", time: "13:51:02", sender: "team@github.com", recipient: "dev@deepmail.io", subject: "[deepmail] PR #142 ready for review", score: 8, sev: "ok", action: "delivered" },
	{ id: "EM-9814", time: "13:48:17", sender: "noreply@office365-login.io", recipient: "admin@deepmail.io", subject: "Microsoft 365: sign-in attempt blocked", score: 88, sev: "critical", action: "blocked" },
	{ id: "EM-9813", time: "13:42:44", sender: "calendar-noreply@google.com", recipient: "vyom@deepmail.io", subject: "Invitation: Sprint Review @ Fri 4pm", score: 4, sev: "ok", action: "delivered" },
];

const ACTION_TONE: Record<Action, "danger" | "warn" | "ok" | "default"> = {
	blocked: "danger",
	quarantined: "warn",
	flagged: "warn",
	delivered: "ok",
};

export default function LogsPage() {
	const [tab, setTab] = useState<Action | "all">("all");
	const [q, setQ] = useState("");

	const counts = useMemo(() => {
		const all = LOGS.length;
		const grouped = LOGS.reduce<Record<Action, number>>(
			(acc, l) => ({ ...acc, [l.action]: (acc[l.action] ?? 0) + 1 }),
			{ delivered: 0, quarantined: 0, blocked: 0, flagged: 0 },
		);
		return { all, ...grouped };
	}, []);

	const filtered = useMemo(() => {
		return LOGS.filter((l) => {
			if (tab !== "all" && l.action !== tab) return false;
			if (q && !`${l.id} ${l.sender} ${l.recipient} ${l.subject}`.toLowerCase().includes(q.toLowerCase())) return false;
			return true;
		});
	}, [tab, q]);

	return (
		<div className="flex flex-col gap-6 w-full max-w-7xl mx-auto">
			<Card
				title="Mail Stream"
				subtitle="Live ingestion log — last 24 hours"
				actions={
					<>
						<Search placeholder="Search…" value={q} onChange={setQ} />
						<Btn size="sm">Export CSV</Btn>
						<Btn size="sm" variant="accent">Live ▶</Btn>
					</>
				}
			>
				<Tabs
					tabs={[
						{ id: "all", label: "All", count: counts.all },
						{ id: "delivered", label: "Delivered", count: counts.delivered },
						{ id: "flagged", label: "Flagged", count: counts.flagged },
						{ id: "quarantined", label: "Quarantined", count: counts.quarantined },
						{ id: "blocked", label: "Blocked", count: counts.blocked },
					]}
					active={tab}
					onChange={(t) => setTab(t as Action | "all")}
				/>

				<div className="overflow-x-auto w-full mt-4">
					<table className="w-full text-sm whitespace-nowrap">
						<thead className="border-b border-border text-muted text-xs">
							<tr>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Time</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Sender</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Recipient</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Subject</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Score</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Severity</th>
								<th className="text-left py-3 px-4 font-medium uppercase tracking-wider">Action</th>
							</tr>
						</thead>
						<tbody>
							{filtered.map((l, i) => (
								<tr key={l.id} className={`border-b border-white/[0.02] hover:bg-surface-2/50 transition-colors ${i % 2 ? "bg-white/[0.01]" : ""}`}>
									<td className="py-3 px-4 font-mono text-[11px] text-muted">{l.time}</td>
									<td className="py-3 px-4 max-w-[200px] truncate text-foreground">{l.sender}</td>
									<td className="py-3 px-4 max-w-[200px] truncate text-muted">{l.recipient}</td>
									<td className="py-3 px-4 max-w-[200px] truncate font-medium text-foreground">{l.subject}</td>
									<td className="py-3 px-4 font-mono text-muted">{l.score}</td>
									<td className="py-3 px-4"><Severity level={l.sev} /></td>
									<td className="py-3 px-4"><Tag tone={ACTION_TONE[l.action]}>{l.action.toUpperCase()}</Tag></td>
								</tr>
							))}
							{filtered.length === 0 && (
								<tr>
									<td colSpan={7} className="text-center py-8 text-muted">
										No log entries match the current filter.
									</td>
								</tr>
							)}
						</tbody>
					</table>
				</div>

				<div className="mt-4 pt-4 border-t border-border flex items-center justify-between text-xs text-muted">
					<span>Showing {filtered.length} of {LOGS.length} entries</span>
					<div className="flex items-center gap-2">
						<Btn size="sm">Prev</Btn>
						<Btn size="sm">Next</Btn>
					</div>
				</div>
			</Card>
		</div>
	);
}
