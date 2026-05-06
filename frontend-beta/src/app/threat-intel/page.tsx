"use client";

import AppShell from "@/components/dashboard/AppShell";
import { BarChart, Btn, Card, Severity, Tag } from "@/components/dashboard/primitives";

type Sev = "critical" | "warning" | "caution" | "ok";

const FEED: { id: string; kind: string; title: string; source: string; when: string; sev: Sev }[] = [
	{ id: "TI-9012", kind: "Campaign", title: "Lazarus group lure targeting fintech HR teams", source: "Mandiant", when: "12m ago", sev: "critical" },
	{ id: "TI-9011", kind: "Indicator", title: "New C2 infra: 198.51.100.42 (Cobalt Strike)", source: "AlienVault OTX", when: "27m ago", sev: "critical" },
	{ id: "TI-9010", kind: "TTP", title: "ClickFix social engineering technique on the rise", source: "Proofpoint", when: "45m ago", sev: "warning" },
	{ id: "TI-9009", kind: "Vulnerability", title: "CVE-2026-1841 — Outlook RCE via crafted .ics", source: "MSRC", when: "1h ago", sev: "warning" },
	{ id: "TI-9008", kind: "Indicator", title: "Phishing kit fingerprint: paypa1-secure.io", source: "URLhaus", when: "2h ago", sev: "warning" },
	{ id: "TI-9007", kind: "Campaign", title: "QR-code phishing surge in EU healthcare", source: "Internal", when: "3h ago", sev: "caution" },
	{ id: "TI-9006", kind: "Report", title: "Q2 2026 phishing trends — quarterly digest", source: "Recorded Future", when: "6h ago", sev: "ok" },
];

const IOCS: { type: string; value: string; hits: number; tone: "danger" | "warn" | "default" }[] = [
	{ type: "Domain", value: "paypa1-secure.io", hits: 47, tone: "danger" },
	{ type: "IP", value: "198.51.100.42", hits: 31, tone: "danger" },
	{ type: "SHA256", value: "a1b2…f7c9", hits: 18, tone: "warn" },
	{ type: "Domain", value: "office365-login.io", hits: 14, tone: "warn" },
	{ type: "URL", value: "/wp-admin/login.php?id=…", hits: 9, tone: "warn" },
	{ type: "Email", value: "ceo@acme-corp.tk", hits: 6, tone: "default" },
];

const FAMILY_BARS: { label: string; value: number; tone?: "danger" | "warn" | "default" }[] = [
	{ label: "Phish", value: 184, tone: "danger" },
	{ label: "BEC", value: 96, tone: "danger" },
	{ label: "Mal", value: 71, tone: "warn" },
	{ label: "Spoof", value: 54, tone: "warn" },
	{ label: "Recon", value: 42 },
	{ label: "Spam", value: 38 },
];

export default function ThreatIntelPage() {
	return (
		<AppShell title="Threat Intelligence" breadcrumb="DEEPMAIL · INTEL">
			<div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
				<Card
					className="lg:col-span-2"
					title="Live Intel Feed"
					subtitle="Aggregated from internal + external sources"
					actions={
						<>
							<Tag>ALL</Tag>
							<Btn size="sm">Filters</Btn>
							<Btn size="sm" variant="accent">Subscribe</Btn>
						</>
					}
				>
					<ul className="divide-y divide-white/[0.04]">
						{FEED.map((f) => (
							<li key={f.id} className="hf-feed-item">
								<div className="flex items-center gap-2 mb-1 flex-wrap">
									<span className="text-[10px] font-mono opacity-50">{f.id}</span>
									<Tag>{f.kind.toUpperCase()}</Tag>
									<Severity level={f.sev} />
									<span className="ml-auto text-[10px] opacity-50">{f.when}</span>
								</div>
								<h4 className="text-sm font-medium leading-snug">{f.title}</h4>
								<p className="text-[11px] opacity-60 mt-1">
									Source · <span className="font-mono">{f.source}</span>
								</p>
							</li>
						))}
					</ul>
				</Card>

				<div className="space-y-6">
					<Card title="Active IOCs" subtitle="Top hits — last 24h">
						<ul className="space-y-2">
							{IOCS.map((i) => (
								<li
									key={i.value}
									className={`hf-ioc-item ${i.tone === "danger" ? "hf-ioc-danger" : i.tone === "warn" ? "hf-ioc-warn" : ""}`}
								>
									<span className="hf-ioc-label">{i.type}</span>
									<span className="font-mono text-xs truncate flex-1 mx-2">{i.value}</span>
									<span className="hf-ioc-count">{i.hits}</span>
								</li>
							))}
						</ul>
					</Card>

					<Card title="Threat Family Mix">
						<BarChart data={FAMILY_BARS} height={180} />
					</Card>
				</div>
			</div>
		</AppShell>
	);
}
