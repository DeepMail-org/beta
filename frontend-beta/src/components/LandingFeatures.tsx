"use client";
import React from "react";
import {
	Shield,
	Globe,
	BarChart2,
	Lock,
	Zap,
	Terminal,
	ArrowRight,
} from "lucide-react";

export function LandingFeatures() {
	const features = [
		{
			title: "IOC Extraction",
			icon: <Shield strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "Extracts IPs, domains, URLs, and file hashes from .eml/.msg. Zero configuration. Multi-layer magic-byte MIME validation.",
		},
		{
			title: "Geo-Intel Enrichment",
			icon: <Globe strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "MaxMind GeoLite2 + AbuseIPDB. Country, ASN, TOR/proxy flags, abuse confidence — cached in Redis with TTL.",
		},
		{
			title: "Threat Scoring",
			icon: <BarChart2 strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "Multi-dimensional scoring engine. VirusTotal for URL/domain reputation. Circuit breakers and exponential retry budgets.",
		},
		{
			title: "Quarantine & Safety",
			icon: <Lock strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "UUID-renamed, 0o400 quarantined. Canonical path validation. No execution permissions. SQL via prepared statements only.",
		},
		{
			title: "Redis Job Queue",
			icon: <Zap strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "Async pipeline via Redis Streams. Upload and analysis fully decoupled. WebSocket notifications on completion.",
		},
		{
			title: "Axum REST API",
			icon: <Terminal strokeWidth={1.5} className="w-[18px] h-[18px]" />,
			desc: "Per-IP token-bucket rate limiting. Token-based auth with rotation scripts. Full REST interface — upload, query, results.",
		},
	];

	return (
		<section
			id="features"
			className="py-24 lg:py-32 px-6 lg:px-10 max-w-[1120px] mx-auto w-full relative z-10"
		>
			<div className="mb-16">
				<p className="text-[11px] font-medium tracking-[0.16em] uppercase text-(--muted) mb-3">
					Architecture
				</p>
				<h2 className="font-display font-semibold text-[clamp(36px,4vw,54px)] text-gradient tracking-[-0.025em] leading-[1.1] whitespace-pre-line">
					{"Built for analysts,\nnot afterthoughts."}
				</h2>
				<p className="font-body font-light text-[17px] text-(--muted) max-w-[520px] mt-4 leading-[1.6]">
					Engineered from the ground up for speed, safety, and deep
					context. DeepMail acts as your automated tier-1 triage
					layer.
				</p>
			</div>

			<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-px bg-(--border) rounded-2xl overflow-hidden glass-shine border border-(--border)">
				{features.map((f, i) => (
					<div
						key={i}
						className="bg-(--surface) p-8 hover:bg-(--surface-2) transition-colors duration-200 group flex flex-col items-start relative h-full"
					>
						<div className="w-10 h-10 glass-shine rounded-[10px] bg-(--glass) flex items-center justify-center text-(--muted) group-hover:text-(--secondary) transition-colors">
							{f.icon}
						</div>

						<h3 className="font-display font-medium text-[15.5px] text-(--foreground) mt-6 mb-2">
							{f.title}
						</h3>

						<p className="font-body font-light text-[13.5px] text-(--muted) leading-[1.65]">
							{f.desc}
						</p>

						<div className="mt-6 mt-auto self-end opacity-0 group-hover:opacity-100 transition-opacity -translate-x-2 group-hover:translate-x-0 transition-transform">
							<ArrowRight className="w-4 h-4 text-(--muted)" />
						</div>
					</div>
				))}
			</div>
		</section>
	);
}
