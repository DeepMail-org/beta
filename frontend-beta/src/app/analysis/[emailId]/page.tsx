"use client";

import { useParams } from "next/navigation";
import Link from "next/link";
import { formatUtcDateTime, formatUtcTime } from "@/lib/format";

// Demo data for the analysis detail view
const DEMO_REPORT = {
	email: {
		id: "demo-001",
		original_name: "invoice_urgent_9921.eml",
		sha256_hash:
			"4f1a8b2c9d3e7f6a5b4c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f",
		file_size: 43827,
		submitted_at: "2026-03-29T12:45:00Z",
		status: "completed" as const,
		completed_at: "2026-03-29T12:45:42Z",
	},
	threat_score: {
		total: 87,
		confidence: 94,
		breakdown: {
			identity: 72,
			infrastructure: 91,
			content: 88,
			attachment: 65,
		},
	},
	headers: {
		spf: {
			result: "fail",
			detail: "SPF record mismatch: domain spoofing detected",
		},
		dkim: {
			result: "fail",
			detail: "DKIM signature invalid: header tampered",
		},
		dmarc: {
			result: "none",
			detail: "No DMARC policy published for sender domain",
		},
		from: "support@micros0ft-billing.com",
		reply_to: "harvester@darkweb-node.onion",
		return_path: "bounce@vps-223.hostingprovider.xyz",
		received_chain: [
			"mail-gateway.example.com",
			"smtp-relay-77.hostingprovider.xyz",
			"vps-223.hostingprovider.xyz",
		],
	},
	iocs: [
		{
			id: "1",
			ioc_type: "domain",
			value: "micros0ft-billing.com",
			first_seen: "2026-03-29T12:45:10Z",
		},
		{
			id: "2",
			ioc_type: "ip",
			value: "201.44.112.9",
			first_seen: "2026-03-29T12:45:10Z",
		},
		{
			id: "3",
			ioc_type: "url",
			value: "https://microsooft-login-secure.xyz/portal/auth",
			first_seen: "2026-03-29T12:45:15Z",
		},
		{
			id: "4",
			ioc_type: "email",
			value: "harvester@darkweb-node.onion",
			first_seen: "2026-03-29T12:45:10Z",
		},
		{
			id: "5",
			ioc_type: "domain",
			value: "vps-223.hostingprovider.xyz",
			first_seen: "2026-03-29T12:45:12Z",
		},
		{
			id: "6",
			ioc_type: "sha256",
			value: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
			first_seen: "2026-03-29T12:45:20Z",
		},
	],
	phishing_keywords: [
		{
			keyword: "urgent action required",
			score: 0.95,
			context: "Subject line manipulation",
		},
		{
			keyword: "verify your account",
			score: 0.92,
			context: "Body text phishing trigger",
		},
		{
			keyword: "billing suspension",
			score: 0.88,
			context: "Social engineering pressure",
		},
		{
			keyword: "click here immediately",
			score: 0.85,
			context: "Call-to-action manipulation",
		},
	],
	urls: [
		{
			url: "https://t.co/redirect/u87v2X",
			status: "redirects",
			risk: "high",
			hops: 3,
		},
		{
			url: "https://microsooft-login-secure.xyz/portal/auth",
			status: "phishing",
			risk: "critical",
			hops: 0,
		},
	],
	attachments: [
		{
			name: "Invoice_March_2026.html",
			type: "text/html",
			size: 8742,
			entropy: 7.2,
			verdict: "malicious",
		},
	],
};

export default function AnalysisDetailPage() {
	const params = useParams();
	const emailId = params.emailId as string;
	const report = DEMO_REPORT;
	const score = report.threat_score;

	const getScoreColor = (s: number) => {
		if (s >= 70) return "text-dracula-red";
		if (s >= 40) return "text-dracula-orange";
		return "text-tertiary";
	};

	const getScoreBadge = (s: number) => {
		if (s >= 70)
			return {
				label: "MALICIOUS",
				bg: "bg-dracula-red/10",
				border: "border-dracula-red/20",
				text: "text-dracula-red",
			};
		if (s >= 40)
			return {
				label: "SUSPICIOUS",
				bg: "bg-dracula-orange/10",
				border: "border-dracula-orange/20",
				text: "text-dracula-orange",
			};
		return {
			label: "SECURE",
			bg: "bg-tertiary/10",
			border: "border-tertiary/20",
			text: "text-tertiary",
		};
	};

	const badge = getScoreBadge(score.total);

	const iocIcons: Record<string, string> = {
		ip: "dns",
		domain: "language",
		url: "link",
		email: "alternate_email",
		md5: "fingerprint",
		sha1: "fingerprint",
		sha256: "fingerprint",
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
				<span className="text-on-surface font-bold">
					Analysis Detail
				</span>
			</div>

			{/* Email Summary Header */}
			<section className="glass-panel rounded-xl p-8">
				<div className="flex flex-col lg:flex-row justify-between gap-8">
					<div className="flex items-start gap-6">
						<div className="w-16 h-16 rounded-xl gradient-primary flex items-center justify-center shadow-lg">
							<span className="material-symbols-outlined text-3xl text-on-primary">
								drafts
							</span>
						</div>
						<div>
							<h2 className="text-2xl font-bold text-on-surface tracking-tight font-headline">
								{report.email.original_name}
							</h2>
							<p className="text-xs text-outline font-mono mt-1 break-all">
								SHA-256: {report.email.sha256_hash}
							</p>
							<div className="flex items-center gap-4 mt-3">
								<span
									className={`${badge.bg} ${badge.text} text-[10px] px-3 py-1 rounded border ${badge.border} font-bold uppercase`}
								>
									{badge.label}
								</span>
								<span className="text-[10px] text-outline">
									{formatUtcDateTime(
										report.email.submitted_at,
									)}
								</span>
								<span className="text-[10px] text-outline">
									{(report.email.file_size / 1024).toFixed(1)}{" "}
									KB
								</span>
							</div>
						</div>
					</div>

					{/* Threat Score Gauge */}
					<div className="flex items-center gap-8">
						<div className="text-center">
							<div className="relative w-28 h-28">
								<svg
									className="w-full h-full -rotate-90"
									viewBox="0 0 120 120"
								>
									<circle
										cx="60"
										cy="60"
										r="50"
										fill="none"
										stroke="#202535"
										strokeWidth="8"
									/>
									<circle
										cx="60"
										cy="60"
										r="50"
										fill="none"
										stroke={
											score.total >= 70
												? "#fd77c4"
												: score.total >= 40
													? "#bd93f9"
													: "#50fa7b"
										}
										strokeWidth="8"
										strokeLinecap="round"
										strokeDasharray={`${(score.total / 100) * 314} 314`}
										className="transition-all duration-1000"
									/>
								</svg>
								<div className="absolute inset-0 flex flex-col items-center justify-center">
									<span
										className={`text-3xl font-bold font-headline ${getScoreColor(score.total)}`}
									>
										{score.total}
									</span>
									<span className="text-[10px] text-outline uppercase">
										/100
									</span>
								</div>
							</div>
							<p className="text-[10px] text-outline mt-2 uppercase tracking-widest font-bold">
								Threat Score
							</p>
						</div>
						<div className="space-y-2">
							{Object.entries(score.breakdown).map(
								([key, val]) => (
									<div
										key={key}
										className="flex items-center gap-3"
									>
										<span className="text-[10px] uppercase tracking-wider text-on-surface-variant w-28 text-right font-bold">
											{key}
										</span>
										<div className="w-32 h-1.5 bg-surface-container-highest rounded-full overflow-hidden">
											<div
												className="h-full rounded-full transition-all duration-1000"
												style={{
													width: `${val}%`,
													background:
														val >= 70
															? "#fd77c4"
															: val >= 40
																? "#bd93f9"
																: "#50fa7b",
												}}
											/>
										</div>
										<span
											className={`text-xs font-bold ${getScoreColor(val)}`}
										>
											{val}
										</span>
									</div>
								),
							)}
						</div>
					</div>
				</div>
			</section>

			{/* Content Grid */}
			<div className="grid grid-cols-12 gap-8">
				{/* Header Analysis */}
				<div className="col-span-12 lg:col-span-6">
					<div className="glass-panel rounded-xl p-6 h-full">
						<h3 className="text-sm font-bold tracking-widest uppercase mb-6 flex items-center gap-2 font-headline">
							<span className="material-symbols-outlined text-primary text-lg">
								policy
							</span>
							Header Analysis
						</h3>
						<div className="space-y-4">
							{/* SPF/DKIM/DMARC */}
							{[
								{ label: "SPF", ...report.headers.spf },
								{ label: "DKIM", ...report.headers.dkim },
								{ label: "DMARC", ...report.headers.dmarc },
							].map((check) => {
								const isPass = check.result === "pass";
								const isFail = check.result === "fail";
								return (
									<div
										key={check.label}
										className="flex items-start gap-3 p-3 bg-surface-container-low/50 rounded-lg"
									>
										<span
											className={`material-symbols-outlined text-sm ${isFail ? "text-error" : isPass ? "text-tertiary" : "text-primary-container"}`}
										>
											{isFail
												? "cancel"
												: isPass
													? "check_circle"
													: "help"}
										</span>
										<div>
											<div className="flex items-center gap-2">
												<span className="text-xs font-bold text-on-surface">
													{check.label}
												</span>
												<span
													className={`text-[10px] px-2 py-0.5 rounded font-bold uppercase ${
														isFail
															? "bg-error/10 text-error border border-error/20"
															: isPass
																? "bg-tertiary/10 text-tertiary border border-tertiary/20"
																: "bg-primary-container/10 text-primary-container border border-primary-container/20"
													}`}
												>
													{check.result}
												</span>
											</div>
											<p className="text-[10px] text-on-surface-variant mt-1">
												{check.detail}
											</p>
										</div>
									</div>
								);
							})}
							{/* Key headers */}
							<div className="border-t border-outline-variant/10 pt-4 space-y-2">
								<HeaderRow
									label="From"
									value={report.headers.from}
									danger
								/>
								<HeaderRow
									label="Reply-To"
									value={report.headers.reply_to}
									danger
								/>
								<HeaderRow
									label="Return-Path"
									value={report.headers.return_path}
								/>
							</div>
						</div>
					</div>
				</div>

				{/* Phishing Keywords */}
				<div className="col-span-12 lg:col-span-6">
					<div className="glass-panel rounded-xl p-6 h-full">
						<h3 className="text-sm font-bold tracking-widest uppercase mb-6 flex items-center gap-2 font-headline">
							<span className="material-symbols-outlined text-dracula-red text-lg">
								phishing
							</span>
							Phishing Indicators
						</h3>
						<div className="space-y-3">
							{report.phishing_keywords.map((kw, i) => (
								<div
									key={i}
									className="flex items-center gap-4 p-3 bg-surface-container-low/50 rounded-lg"
								>
									<div className="w-10 h-10 rounded-lg bg-dracula-red/10 flex items-center justify-center border border-dracula-red/20">
										<span className="text-sm font-bold text-dracula-red">
											{Math.round(kw.score * 100)}%
										</span>
									</div>
									<div className="flex-1">
										<p className="text-xs font-bold text-on-surface">
											&ldquo;{kw.keyword}&rdquo;
										</p>
										<p className="text-[10px] text-on-surface-variant">
											{kw.context}
										</p>
									</div>
									<div className="w-20 h-1.5 bg-surface-container-highest rounded-full overflow-hidden">
										<div
											className="h-full bg-dracula-red rounded-full"
											style={{
												width: `${kw.score * 100}%`,
											}}
										/>
									</div>
								</div>
							))}
						</div>

						{/* URL Analysis */}
						<h4 className="text-xs font-bold tracking-widest uppercase mt-8 mb-4 text-on-surface-variant">
							URL_ANALYSIS
						</h4>
						<div className="space-y-3">
							{report.urls.map((u, i) => (
								<div
									key={i}
									className={`p-3 rounded-lg flex items-start gap-3 ${u.risk === "critical" ? "bg-error/5 border border-error/20" : "bg-surface-container-low/50"}`}
								>
									<span
										className={`material-symbols-outlined text-sm ${u.risk === "critical" ? "text-error" : "text-dracula-orange"}`}
									>
										{u.risk === "critical"
											? "warning"
											: "alt_route"}
									</span>
									<div className="flex-1 min-w-0">
										<p className="text-[10px] font-mono text-on-surface break-all">
											{u.url}
										</p>
										<div className="flex items-center gap-3 mt-1">
											<span
												className={`text-[10px] font-bold uppercase ${u.risk === "critical" ? "text-error" : "text-dracula-orange"}`}
											>
												{u.status}
											</span>
											{u.hops > 0 && (
												<span className="text-[10px] text-outline">
													{u.hops} redirects
												</span>
											)}
										</div>
									</div>
								</div>
							))}
						</div>
					</div>
				</div>

				{/* IOC Table */}
				<div className="col-span-12">
					<div className="glass-panel rounded-xl overflow-hidden">
						<div className="px-6 py-5 border-b border-outline-variant/10 flex justify-between items-center bg-surface-container-low/50">
							<h3 className="text-sm font-bold tracking-widest uppercase flex items-center gap-2 font-headline">
								<span className="material-symbols-outlined text-primary text-lg">
									search
								</span>
								Indicators of Compromise ({report.iocs.length})
							</h3>
							<div className="flex gap-2">
								<Link
									href={`/analysis/${emailId}/graph`}
									className="text-[10px] font-bold text-tertiary hover:underline flex items-center gap-1"
								>
									<span className="material-symbols-outlined text-xs">
										hub
									</span>
									GRAPH_VIEW
								</Link>
							</div>
						</div>
						<div className="overflow-x-auto">
							<table className="w-full text-left">
								<thead className="text-[10px] uppercase tracking-widest text-outline bg-bg/40">
									<tr>
										<th className="px-6 py-3 font-bold">
											Type
										</th>
										<th className="px-6 py-3 font-bold">
											Value
										</th>
										<th className="px-6 py-3 font-bold">
											First Seen
										</th>
										<th className="px-6 py-3 font-bold">
											Actions
										</th>
									</tr>
								</thead>
								<tbody className="text-sm">
									{report.iocs.map((ioc) => (
										<tr
											key={ioc.id}
											className="hover:bg-surface-container-high/40 transition-colors border-b border-outline-variant/5"
										>
											<td className="px-6 py-3">
												<div className="flex items-center gap-2">
													<span className="material-symbols-outlined text-primary text-sm">
														{iocIcons[
															ioc.ioc_type
														] || "tag"}
													</span>
													<span className="text-[10px] font-bold uppercase text-on-surface-variant bg-surface-container px-2 py-0.5 rounded">
														{ioc.ioc_type}
													</span>
												</div>
											</td>
											<td className="px-6 py-3 font-mono text-xs text-on-surface break-all max-w-md">
												{ioc.value}
											</td>
											<td className="px-6 py-3 text-[10px] text-outline">
												{formatUtcTime(ioc.first_seen)}
											</td>
											<td className="px-6 py-3 relative group/action">
												<button className="material-symbols-outlined text-outline hover:text-on-surface text-lg">
													more_vert
												</button>
												<div className="absolute right-6 top-full mt-1 w-40 bg-surface-container-high border border-white/10 rounded-lg shadow-2xl opacity-0 invisible group-hover/action:opacity-100 group-hover/action:visible transition-all z-20 flex flex-col overflow-hidden">
													<button className="px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-left hover:bg-primary/10 hover:text-primary flex items-center gap-2 transition-colors">
														<span className="material-symbols-outlined text-sm">
															search
														</span>
														Deep Search
													</button>
													<button className="px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-left hover:bg-primary/10 hover:text-primary flex items-center gap-2 transition-colors">
														<span className="material-symbols-outlined text-sm">
															content_copy
														</span>
														Copy Value
													</button>
													<button className="px-4 py-2 text-[10px] font-bold uppercase tracking-widest text-left hover:bg-error/10 text-error flex items-center gap-2 transition-colors">
														<span className="material-symbols-outlined text-sm">
															block
														</span>
														Blacklist
													</button>
												</div>
											</td>
										</tr>
									))}
								</tbody>
							</table>
						</div>
					</div>
				</div>

				{/* Attachments */}
				<div className="col-span-12 lg:col-span-6">
					<div className="glass-panel rounded-xl p-6">
						<h3 className="text-sm font-bold tracking-widest uppercase mb-4 flex items-center gap-2 font-headline">
							<span className="material-symbols-outlined text-error text-lg">
								attachment
							</span>
							Attachments
						</h3>
						{report.attachments.map((att, i) => (
							<div
								key={i}
								className="p-4 bg-error/5 border border-error/20 rounded-lg flex items-center gap-4"
							>
								<div className="w-12 h-12 rounded-lg bg-error/10 flex items-center justify-center border border-error/20">
									<span className="material-symbols-outlined text-error text-xl">
										description
									</span>
								</div>
								<div className="flex-1">
									<p className="text-sm font-bold text-on-surface">
										{att.name}
									</p>
									<div className="flex items-center gap-3 mt-1">
										<span className="text-[10px] text-outline">
											{att.type}
										</span>
										<span className="text-[10px] text-outline">
											{(att.size / 1024).toFixed(1)} KB
										</span>
										<span className="text-[10px] text-error font-bold">
											Entropy: {att.entropy}
										</span>
									</div>
								</div>
								<span className="bg-error/10 text-error text-[10px] px-2 py-1 rounded border border-error/20 font-bold uppercase">
									{att.verdict}
								</span>
							</div>
						))}
					</div>
				</div>

				{/* Quick Actions */}
				<div className="col-span-12 lg:col-span-6">
					<div className="glass-panel rounded-xl p-6 h-full flex flex-col justify-between">
						<div>
							<h3 className="text-sm font-bold tracking-widest uppercase mb-4 flex items-center gap-2 font-[family-name:var(--font-headline)]">
								<span className="material-symbols-outlined text-primary text-lg">
									bolt
								</span>
								Quick Actions
							</h3>
							<div className="grid grid-cols-2 gap-3">
								<Link
									href={`/analysis/${emailId}/terminal`}
									className="p-4 bg-surface-container-high rounded-lg border border-outline-variant/10 hover:border-primary/30 transition-all flex flex-col items-center gap-2 text-center"
								>
									<span className="material-symbols-outlined text-primary text-2xl">
										terminal
									</span>
									<span className="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant">
										Terminal View
									</span>
								</Link>
								<Link
									href={`/analysis/${emailId}/graph`}
									className="p-4 bg-surface-container-high rounded-lg border border-outline-variant/10 hover:border-tertiary/30 transition-all flex flex-col items-center gap-2 text-center"
								>
									<span className="material-symbols-outlined text-tertiary text-2xl">
										hub
									</span>
									<span className="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant">
										Graph View
									</span>
								</Link>
								<button className="p-4 bg-surface-container-high rounded-lg border border-outline-variant/10 hover:border-dracula-red/30 transition-all flex flex-col items-center gap-2 text-center">
									<span className="material-symbols-outlined text-dracula-red text-2xl">
										download
									</span>
									<span className="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant">
										Export PDF
									</span>
								</button>
								<button className="p-4 bg-surface-container-high rounded-lg border border-outline-variant/10 hover:border-primary/30 transition-all flex flex-col items-center gap-2 text-center">
									<span className="material-symbols-outlined text-primary text-2xl">
										share
									</span>
									<span className="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant">
										Share Report
									</span>
								</button>
							</div>
						</div>
						<div className="mt-6 p-4 bg-tertiary/5 border border-tertiary/20 rounded-lg flex items-start gap-3">
							<span className="material-symbols-outlined text-tertiary text-sm">
								info
							</span>
							<p className="text-[10px] leading-relaxed text-tertiary/80">
								Analysis completed in{" "}
								<strong>42 seconds</strong>. All 8 pipeline
								stages executed successfully with 94%
								confidence.
							</p>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}

function HeaderRow({
	label,
	value,
	danger,
}: {
	label: string;
	value: string;
	danger?: boolean;
}) {
	return (
		<div className="flex items-center gap-3">
			<span className="text-[10px] font-bold uppercase text-outline w-24 text-right tracking-widest">
				{label}:
			</span>
			<span
				className={`text-xs font-mono ${danger ? "text-dracula-red" : "text-on-surface-variant"}`}
			>
				{value}
			</span>
		</div>
	);
}
