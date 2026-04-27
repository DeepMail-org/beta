"use client";

import Link from "next/link";
import { useParams } from "next/navigation";

const TERMINAL_LINES = [
	{
		time: "09:12:04.022",
		level: "info",
		module: "pipeline",
		text: "Initializing analysis pipeline for email_id=demo-001",
	},
	{
		time: "09:12:04.045",
		level: "info",
		module: "parser",
		text: "Parsing email: invoice_urgent_9921.eml (42.8 KB)",
	},
	{
		time: "09:12:04.189",
		level: "ok",
		module: "parser",
		text: "Email parsed successfully. Content-Type: multipart/mixed",
	},
	{
		time: "09:12:04.201",
		level: "info",
		module: "headers",
		text: "Starting header analysis...",
	},
	{
		time: "09:12:04.312",
		level: "warn",
		module: "headers",
		text: "SPF check FAILED: sender IP 201.44.112.9 not in SPF record for micros0ft-billing.com",
	},
	{
		time: "09:12:04.315",
		level: "warn",
		module: "headers",
		text: "DKIM check FAILED: signature hash mismatch (header tampered)",
	},
	{
		time: "09:12:04.318",
		level: "warn",
		module: "headers",
		text: "DMARC: No policy published for sender domain",
	},
	{
		time: "09:12:04.320",
		level: "error",
		module: "headers",
		text: "IDENTITY SCORE: 72/100 — Likely spoofed sender",
	},
	{
		time: "09:12:04.450",
		level: "info",
		module: "ioc",
		text: "Extracting IOCs from email body and headers...",
	},
	{
		time: "09:12:05.102",
		level: "ok",
		module: "ioc",
		text: "6 IOCs extracted: 2 domains, 2 IPs, 1 URL, 1 email address",
	},
	{
		time: "09:12:05.230",
		level: "info",
		module: "phishing",
		text: "Running phishing keyword analysis...",
	},
	{
		time: "09:12:05.567",
		level: "error",
		module: "phishing",
		text: 'PHISHING SCORE: 0.95 — "urgent action required" detected in subject',
	},
	{
		time: "09:12:05.571",
		level: "error",
		module: "phishing",
		text: 'PHISHING SCORE: 0.92 — "verify your account" detected in body',
	},
	{
		time: "09:12:05.575",
		level: "warn",
		module: "phishing",
		text: 'PHISHING SCORE: 0.88 — "billing suspension" social engineering trigger',
	},
	{
		time: "09:12:06.000",
		level: "info",
		module: "url",
		text: "Analyzing 2 URLs found in email...",
	},
	{
		time: "09:12:06.445",
		level: "warn",
		module: "url",
		text: "URL https://t.co/redirect/u87v2X — 3 redirect hops detected",
	},
	{
		time: "09:12:07.891",
		level: "error",
		module: "url",
		text: "🚨 CRITICAL: Final destination is phishing clone — microsooft-login-secure.xyz/portal/auth",
	},
	{
		time: "09:12:08.100",
		level: "info",
		module: "attachment",
		text: "Analyzing 1 attachment: Invoice_March_2026.html (8.5 KB)",
	},
	{
		time: "09:12:08.445",
		level: "error",
		module: "attachment",
		text: "HIGH ENTROPY (7.2): Obfuscated JavaScript found in HTML attachment",
	},
	{
		time: "09:12:09.000",
		level: "info",
		module: "scoring",
		text: "Computing final threat score...",
	},
	{
		time: "09:12:09.231",
		level: "error",
		module: "scoring",
		text: "═══ FINAL VERDICT: 87/100 — MALICIOUS (Confidence: 94%) ═══",
	},
	{
		time: "09:12:09.232",
		level: "info",
		module: "scoring",
		text: "Breakdown: identity=72, infrastructure=91, content=88, attachment=65",
	},
	{
		time: "09:12:09.500",
		level: "ok",
		module: "pipeline",
		text: "Results persisted to database. Analysis complete in 5.478s",
	},
	{
		time: "09:12:09.502",
		level: "ok",
		module: "pipeline",
		text: "✓ Pipeline finished. Email quarantined.",
	},
];

export default function TerminalPage() {
	const params = useParams();
	const emailId = params.emailId as string;

	const levelColors: Record<string, string> = {
		info: "text-primary",
		ok: "text-tertiary",
		warn: "text-dracula-orange",
		error: "text-error",
	};

	const statColorClasses: Record<string, string> = {
		primary: "text-primary",
		tertiary: "text-tertiary",
		"dracula-orange": "text-dracula-orange",
		error: "text-error",
	};

	return (
		<div className="p-8 lg:p-12 space-y-8 max-w-7xl mx-auto">
			{/* Breadcrumb */}
			<div className="flex items-center gap-2 text-xs text-on-surface-variant">
				<Link href="/" className="hover:text-primary transition-colors">
					Dashboard
				</Link>
				<span className="material-symbols-outlined text-xs">
					chevron_right
				</span>
				<Link
					href={`/analysis/${emailId}`}
					className="hover:text-primary transition-colors"
				>
					Analysis
				</Link>
				<span className="material-symbols-outlined text-xs">
					chevron_right
				</span>
				<span className="text-on-surface font-bold">Terminal</span>
			</div>

			{/* Header */}
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-3xl font-bold text-on-surface tracking-tight font-[family-name:var(--font-headline)]">
						Pipeline <span className="text-primary">Terminal</span>
					</h2>
					<p className="text-xs text-on-surface-variant mt-1">
						Raw pipeline output log
					</p>
				</div>
				<div className="flex gap-3">
					<button className="px-4 py-2 bg-surface-container-high border border-outline-variant/20 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:border-primary/30 transition-all flex items-center gap-2">
						<span className="material-symbols-outlined text-sm">
							content_copy
						</span>
						Copy Log
					</button>
					<button className="px-4 py-2 bg-surface-container-high border border-outline-variant/20 rounded-lg text-[10px] font-bold uppercase tracking-widest hover:border-primary/30 transition-all flex items-center gap-2">
						<span className="material-symbols-outlined text-sm">
							download
						</span>
						Export
					</button>
				</div>
			</div>

			{/* Terminal */}
			<div className="glass-panel rounded-xl overflow-hidden">
				{/* Terminal title bar */}
				<div className="bg-surface-container-lowest px-6 py-3 border-b border-outline-variant/10 flex items-center justify-between">
					<div className="flex items-center gap-3">
						<div className="flex gap-2">
							<div className="w-3 h-3 rounded-full bg-error/60" />
							<div className="w-3 h-3 rounded-full bg-dracula-orange/60" />
							<div className="w-3 h-3 rounded-full bg-tertiary/60" />
						</div>
						<span className="text-[10px] font-mono text-outline">
							deepmail-worker — pipeline-output
						</span>
					</div>
					<span className="text-[10px] font-mono text-outline">
						24 lines
					</span>
				</div>

				{/* Terminal content */}
				<div className="bg-surface-container-lowest/80 p-6 font-mono text-xs leading-relaxed overflow-x-auto max-h-[600px] overflow-y-auto no-scrollbar">
					{TERMINAL_LINES.map((line, i) => (
						<div
							key={i}
							className="flex gap-4 py-1 hover:bg-surface-container/30 px-2 -mx-2 rounded transition-colors group"
						>
							<span className="text-outline/40 select-none w-6 text-right shrink-0">
								{i + 1}
							</span>
							<span className="text-outline/60 shrink-0">
								{line.time}
							</span>
							<span
								className={`uppercase font-bold shrink-0 w-12 ${levelColors[line.level]}`}
							>
								{line.level === "ok"
									? " OK "
									: line.level.toUpperCase()}
							</span>
							<span className="text-primary/50 shrink-0">
								[{line.module}]
							</span>
							<span
								className={`${line.level === "error" ? "text-error" : line.level === "warn" ? "text-dracula-orange" : line.level === "ok" ? "text-tertiary/80" : "text-on-surface/70"}`}
							>
								{line.text}
							</span>
						</div>
					))}
					{/* Cursor */}
					<div className="flex gap-4 py-1 px-2">
						<span className="text-outline/40 select-none w-6 text-right">
							{TERMINAL_LINES.length + 1}
						</span>
						<span className="text-tertiary animate-pulse">▊</span>
					</div>
				</div>
			</div>

			{/* Stats Bar */}
			<div className="grid grid-cols-4 gap-4">
				{[
					{
						icon: "timer",
						label: "Duration",
						value: "5.478s",
						color: "primary",
					},
					{
						icon: "check_circle",
						label: "Stages",
						value: "8/8",
						color: "tertiary",
					},
					{
						icon: "warning",
						label: "Warnings",
						value: "5",
						color: "dracula-orange",
					},
					{
						icon: "error",
						label: "Critical",
						value: "6",
						color: "error",
					},
				].map((stat) => (
					<div
						key={stat.label}
						className="glass-panel p-4 rounded-xl flex items-center gap-4"
					>
						<span
							className={`material-symbols-outlined text-2xl ${statColorClasses[stat.color]}`}
						>
							{stat.icon}
						</span>
						<div>
							<p
								className={`text-xl font-bold font-[family-name:var(--font-headline)] ${statColorClasses[stat.color]}`}
							>
								{stat.value}
							</p>
							<p className="text-[10px] uppercase tracking-widest text-outline font-bold">
								{stat.label}
							</p>
						</div>
					</div>
				))}
			</div>
		</div>
	);
}
