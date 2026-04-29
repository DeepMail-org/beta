"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";

const faqs = [
	{
		q: "What does DeepMail do?",
		a: "DeepMail is a multi-layered email threat intelligence engine. It parses raw .eml files, extracts IOCs (IPs, domains, URLs, hashes), runs geo-intelligence enrichment, scores payloads, and delivers a full threat report in seconds.",
	},
	{
		q: "What file types are supported?",
		a: "We support standard .eml (RFC 822) email files. You can upload individual messages or drag-and-drop multiple files at once. Each file is processed through our full pipeline independently.",
	},
	{
		q: "How does the scoring system work?",
		a: "Our engine aggregates signals across PARSE, IOC TRIGGER, GEO-INTEL, URL REG, FILE analysis, and SCORING stages. Each stage contributes weighted sub-scores to a final threat confidence rating from 0–100.",
	},
	{
		q: "Is my data stored after analysis?",
		a: "Email data is processed ephemerally in an isolated sandbox. Results are retained only for the duration of your session and purged on logout. We never store raw email payloads beyond the analysis window.",
	},
	{
		q: "Which threat intel sources do you integrate?",
		a: "DeepMail currently integrates MaxMind for geo-IP, AbuseIPDB for reputation checks, and VirusTotal for file hash and URL lookups — all cached for performance via Redis Streams.",
	},
	{
		q: "Can I use the API directly?",
		a: "Yes. Our REST API exposes endpoints for upload, IOC extraction, and results retrieval. WebSocket support is available for real-time pipeline progress. Contact us for API keys and rate limit tiers.",
	},
];

export default function FAQWithSpiral() {
	const [query, setQuery] = useState("");
	const [openIdx, setOpenIdx] = useState<number | null>(null);

	const filtered = query
		? faqs.filter(({ q, a }) => (q + a).toLowerCase().includes(query.toLowerCase()))
		: faqs;

	return (
		<div className="relative w-full overflow-hidden text-white">
			{/* Subtle radial background glow — CSS only, zero JS cost */}
			<div
				className="pointer-events-none absolute inset-0"
				aria-hidden
				style={{
					background:
						"radial-gradient(ellipse 70% 50% at 50% 60%, rgba(255,255,255,0.03) 0%, transparent 70%)",
				}}
			/>

			<div className="relative mx-auto max-w-5xl px-6 py-16">
				{/* Header */}
				<header className="mb-10 flex items-end justify-between border-b border-white/10 pb-6 gap-4 flex-wrap">
					<h1 className="text-4xl md:text-6xl font-black tracking-tight">FAQ</h1>
					<input
						value={query}
						onChange={(e) => setQuery(e.target.value)}
						placeholder="Search questions…"
						className="h-10 w-56 rounded-xl border border-white/15 bg-white/3 px-3 text-sm outline-none transition-all duration-200 focus:border-white/35 focus:bg-white/5 placeholder:text-white/25"
					/>
				</header>

				{/* FAQ items */}
				<div className="grid grid-cols-1 gap-2 md:grid-cols-2">
					<AnimatePresence>
						{filtered.map((item, i) => (
							<FAQItem
								key={item.q}
								q={item.q}
								a={item.a}
								index={i + 1}
								isOpen={openIdx === i}
								onToggle={() => setOpenIdx(openIdx === i ? null : i)}
							/>
						))}
					</AnimatePresence>

					{filtered.length === 0 && (
						<p className="col-span-full text-center text-sm text-white/30 py-10">
							No questions match &ldquo;{query}&rdquo;
						</p>
					)}
				</div>
			</div>
		</div>
	);
}

function FAQItem({
	q,
	a,
	index,
	isOpen,
	onToggle,
}: {
	q: string;
	a: string;
	index: number;
	isOpen: boolean;
	onToggle: () => void;
}) {
	return (
		<div
			className={`rounded-2xl border transition-colors duration-200 ${
				isOpen ? "border-white/20 bg-white/4" : "border-white/10 bg-white/2 hover:border-white/18"
			}`}
		>
			<button
				onClick={onToggle}
				className="flex w-full items-center justify-between gap-4 p-5 text-left"
				aria-expanded={isOpen}
			>
				<div className="flex items-baseline gap-3 min-w-0">
					<span className="shrink-0 text-xs tabular-nums text-white/25">
						{String(index).padStart(2, "0")}
					</span>
					<h3 className="text-sm md:text-base font-semibold leading-snug">{q}</h3>
				</div>
				<motion.span
					animate={{ rotate: isOpen ? 45 : 0 }}
					transition={{ duration: 0.2, ease: "easeInOut" }}
					className="shrink-0 text-xl leading-none text-white/50"
				>
					+
				</motion.span>
			</button>

			<AnimatePresence initial={false}>
				{isOpen && (
					<motion.div
						key="answer"
						initial={{ height: 0, opacity: 0 }}
						animate={{ height: "auto", opacity: 1 }}
						exit={{ height: 0, opacity: 0 }}
						transition={{ duration: 0.25, ease: [0.4, 0, 0.2, 1] }}
						className="overflow-hidden"
					>
						<p className="px-5 pb-5 text-sm text-white/60 leading-relaxed">
							{a}
						</p>
					</motion.div>
				)}
			</AnimatePresence>
		</div>
	);
}
