"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Plus, HelpCircle } from "lucide-react";

const faqs = [
	{
		q: "What does DeepMail do?",
		a: "DeepMail is a multi-layered email threat intelligence engine. It parses raw .eml files, extracts IOCs (IPs, domains, URLs, hashes), runs geo-intelligence enrichment, scores payloads and delivers a full threat report in seconds.",
	},
	{
		q: "What file types are supported?",
		a: "We support standard .eml (RFC 822) email files. You can upload individual messages or drag-and-drop multiple files at once. Each file is processed through our full pipeline independently.",
	},
	{
		q: "How does the scoring system work?",
		a: "Our engine aggregates signals across PARSE, IOC TRIGGER, GEO-INTEL, URL REG, FILE analysis and SCORING stages. Each stage contributes weighted sub-scores to a final threat confidence rating from 0–100.",
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
		a: "Yes. Our REST API exposes endpoints for upload, IOC extraction and results retrieval. WebSocket support is available for real-time pipeline progress. Contact us for API keys and rate limit tiers.",
	},
];

export default function FAQWithSpiral() {
	const [openIdx, setOpenIdx] = useState<number | null>(0);

	return (
		<section className="relative w-full overflow-hidden">
			{/* Ambient backdrop */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-0"
				style={{
					background:
						"radial-gradient(ellipse 80% 60% at 50% 0%, rgba(255,255,255,0.04) 0%, transparent 70%)",
				}}
			/>
			<div
				aria-hidden
				className="pointer-events-none absolute inset-x-0 top-0 h-px"
				style={{
					background:
						"linear-gradient(90deg, transparent, rgba(255,255,255,0.15), transparent)",
				}}
			/>

			<div className="relative mx-auto max-w-3xl px-6 py-20">
				{/* Header */}
				<header className="mb-14 flex flex-col items-center gap-4 text-center">
					<div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/3 px-3 py-1">
						<HelpCircle className="h-3 w-3 text-white/55" strokeWidth={1.5} />
						<span className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/55">
							Frequently asked
						</span>
					</div>
					<h2 className="font-display text-4xl font-semibold tracking-tight text-white md:text-5xl">
						Questions, answered
					</h2>
					<p className="max-w-lg text-sm text-white/40 leading-relaxed">
						Everything analysts ask before they ship DeepMail in front of their SOC. Still curious? Drop us a line.
					</p>
				</header>

				{/* Accordion */}
				<ul className="flex flex-col gap-3">
					{faqs.map((item, i) => {
						const isOpen = openIdx === i;
						return (
							<li
								key={item.q}
								className={`overflow-hidden rounded-2xl border transition-colors duration-200 ${
									isOpen
										? "border-white/20 bg-white/4"
										: "border-white/8 bg-white/2 hover:border-white/15"
								}`}
							>
								<button
									onClick={() => setOpenIdx(isOpen ? null : i)}
									className="flex w-full items-center justify-between gap-6 px-6 py-5 text-left"
									aria-expanded={isOpen}
								>
									<div className="flex items-baseline gap-4 min-w-0">
										<span className="shrink-0 font-mono text-[11px] tabular-nums text-white/30">
											{String(i + 1).padStart(2, "0")}
										</span>
										<h3 className="text-base font-medium text-white/90 leading-snug">
											{item.q}
										</h3>
									</div>
									<motion.span
										animate={{ rotate: isOpen ? 45 : 0 }}
										transition={{ duration: 0.25, ease: "easeOut" }}
										className="shrink-0 text-white/55"
									>
										<Plus className="h-5 w-5" strokeWidth={1.5} />
									</motion.span>
								</button>

								<AnimatePresence initial={false}>
									{isOpen && (
										<motion.div
											key="answer"
											initial={{ height: 0, opacity: 0 }}
											animate={{ height: "auto", opacity: 1 }}
											exit={{ height: 0, opacity: 0 }}
											transition={{ duration: 0.28, ease: [0.4, 0, 0.2, 1] }}
											className="overflow-hidden"
										>
											<div className="border-t border-white/6 px-6 py-5 pl-15">
												<p className="text-sm text-white/55 leading-relaxed">
													{item.a}
												</p>
											</div>
										</motion.div>
									)}
								</AnimatePresence>
							</li>
						);
					})}
				</ul>

				{/* Footer CTA */}
				<div className="mt-12 flex flex-col items-center gap-3 text-center">
					<p className="text-xs text-white/35">
						Didn&apos;t find what you needed?
					</p>
					<a
						href="/contact"
						className="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/3 px-4 py-2 text-xs font-medium text-white/80 transition-all duration-200 hover:border-white/30 hover:bg-white/8"
					>
						Talk to the team
						<span aria-hidden>→</span>
					</a>
				</div>
			</div>
		</section>
	);
}
