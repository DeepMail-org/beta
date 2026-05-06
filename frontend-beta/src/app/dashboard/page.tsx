"use client";

import AppShell from "@/components/dashboard/AppShell";
import {
	Btn,
	Card,
	Donut,
	GeoMap,
	LineChart,
	RadarChart,
	Severity,
	StatCard,
	Tag,
} from "@/components/dashboard/primitives";

const TIMELINE = [
	{ label: "Mon", value: 142 },
	{ label: "Tue", value: 168 },
	{ label: "Wed", value: 199 },
	{ label: "Thu", value: 174 },
	{ label: "Fri", value: 231 },
	{ label: "Sat", value: 184 },
	{ label: "Sun", value: 257 },
];

const SEVERITY_SEGMENTS = [
	{ label: "Critical", value: 14, color: "#f87171" },
	{ label: "High", value: 27, color: "#fbbf24" },
	{ label: "Medium", value: 41, color: "#facc15" },
	{ label: "Safe", value: 218, color: "rgba(255,255,255,0.55)" },
];

const VECTOR_AXES = [
	{ label: "Phishing", value: 0.92 },
	{ label: "Malware", value: 0.64 },
	{ label: "BEC", value: 0.78 },
	{ label: "Spoofing", value: 0.55 },
	{ label: "Spam", value: 0.42 },
	{ label: "Recon", value: 0.31 },
];

const TOP_SENDERS = [
	{ rank: 1, sender: "no-reply@apple-id.support", count: 47 },
	{ rank: 2, sender: "billing@paypal-secure.io", count: 38 },
	{ rank: 3, sender: "ceo@acme-corp.tk", count: 29 },
	{ rank: 4, sender: "alerts@dropbox-share.app", count: 24 },
	{ rank: 5, sender: "noreply@office365-login.io", count: 19 },
];

const HOTSPOTS = [
	{ x: 200, y: 160, intensity: 1.0, label: "RU" },
	{ x: 540, y: 170, intensity: 0.7, label: "CN" },
	{ x: 380, y: 200, intensity: 0.5, label: "IR" },
	{ x: 660, y: 240, intensity: 0.4, label: "ID" },
	{ x: 140, y: 220, intensity: 0.3, label: "BR" },
];

const RECENT_THREATS = [
	{ id: "ALR-7821", subject: "URGENT: Wire transfer authorization required", sender: "cfo@acme-corp.tk", level: "critical" as const, score: 96, when: "2m ago" },
	{ id: "ALR-7820", subject: "Your Apple ID password was reset", sender: "no-reply@apple-id.support", level: "critical" as const, score: 91, when: "9m ago" },
	{ id: "ALR-7819", subject: "Invoice #1224 — overdue", sender: "billing@vendor-portal.app", level: "warning" as const, score: 78, when: "17m ago" },
	{ id: "ALR-7818", subject: "Shared file: Q3-Forecast.xlsx", sender: "alerts@dropbox-share.app", level: "warning" as const, score: 71, when: "24m ago" },
	{ id: "ALR-7817", subject: "Re: Vendor onboarding documents", sender: "ops@known-vendor.com", level: "caution" as const, score: 54, when: "38m ago" },
];

export default function DashboardPage() {
	return (
		<AppShell title="Threat Overview" breadcrumb="DEEPMAIL · OPERATIONS">
			<div className="hf-stats-row hf-fade-stagger">
				<StatCard label="Inbound Today" value="2,847" change="+12.4%" trend="up" />
				<StatCard label="Threats Blocked" value="318" change="+8.2%" trend="up" />
				<StatCard label="Critical Alerts" value="14" change="+2" trend="up" />
				<StatCard label="Open Cases" value="9" change="-3" trend="down" />
				<StatCard label="Detection Rate" value="99.4%" change="+0.3%" trend="up" />
			</div>

			<div className="hf-dash-grid mt-6 hf-fade-stagger">
				<Card
					className="hf-w-3"
					title="Threat Timeline"
					subtitle="Detections over the last 7 days"
					actions={<><Tag>7D</Tag><Btn size="sm">Export</Btn></>}
				>
					<LineChart data={TIMELINE} height={220} />
				</Card>

				<Card className="hf-w-1" title="Severity Mix" subtitle="Last 24h">
					<Donut segments={SEVERITY_SEGMENTS} centerLabel="TOTAL" centerValue="300" />
				</Card>

				<Card className="hf-w-2" title="Attack Vector Profile">
					<div className="flex items-center justify-center">
						<RadarChart axes={VECTOR_AXES} size={260} />
					</div>
				</Card>

				<Card className="hf-w-2" title="Top Suspicious Senders" subtitle="Past 24 hours">
					<ul className="divide-y divide-white/[0.04]">
						{TOP_SENDERS.map((s) => (
							<li key={s.rank} className="hf-list-row">
								<span className="hf-list-rank">{String(s.rank).padStart(2, "0")}</span>
								<span className="hf-list-text truncate">{s.sender}</span>
								<span className="hf-ioc-count">{s.count}</span>
							</li>
						))}
					</ul>
				</Card>

				<Card
					className="hf-w-3"
					title="Geo Threat Origins"
					subtitle="Real-time origin map"
					actions={<span className="flex items-center gap-1.5 text-[11px] opacity-70"><span className="hf-live-dot" /> LIVE</span>}
				>
					<GeoMap hotspots={HOTSPOTS} />
				</Card>

				<Card
					className="hf-w-1"
					title="Recent Threats"
					subtitle="Click to inspect"
					actions={<Btn size="sm">View All</Btn>}
				>
					<ul className="space-y-2">
						{RECENT_THREATS.map((t) => (
							<li key={t.id} className="p-3 rounded-lg bg-white/[0.02] hover:bg-white/[0.05] transition-colors cursor-pointer">
								<div className="flex items-center justify-between gap-2 mb-1">
									<span className="text-[10px] font-mono opacity-50">{t.id}</span>
									<Severity level={t.level} />
								</div>
								<div className="text-xs font-medium truncate">{t.subject}</div>
								<div className="flex items-center justify-between gap-2 mt-1.5 text-[10px] opacity-60">
									<span className="truncate">{t.sender}</span>
									<span>{t.when}</span>
								</div>
							</li>
						))}
					</ul>
				</Card>
			</div>
		</AppShell>
	);
}
