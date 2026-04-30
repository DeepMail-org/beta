"use client";
import React from "react";
import { motion } from "framer-motion";
import { Hash, MessageSquare, AtSign, Code2, Search } from "lucide-react";

export function FeaturesAndBenefits() {
	return (
		<section className="relative w-full overflow-hidden py-20">
			<div
				aria-hidden
				className="pointer-events-none absolute inset-0"
				style={{
					background:
						"radial-gradient(ellipse 70% 50% at 50% 0%, rgba(255,255,255,0.03) 0%, transparent 70%)",
				}}
			/>
			<div className="relative mx-auto max-w-6xl px-6">
				{/* Heading */}
				<header className="mb-12 text-center">
					<p className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/40">
						Features &amp; benefits
					</p>
					<h2 className="mt-3 font-display text-3xl md:text-5xl font-semibold tracking-tight text-white">
						Adopted by analysts. Loved by SOCs.
					</h2>
				</header>

				{/* Two columns */}
				<div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
					<UserAdoptionCard />
					<TestimonialCard />
				</div>
			</div>
		</section>
	);
}

// ============================================================
// LEFT CARD — Major User Adoption
// ============================================================
function UserAdoptionCard() {
	const integrations = [
		{ Icon: MessageSquare, label: "Slack" },
		{ Icon: Hash, label: "Splunk" },
		{ Icon: AtSign, label: "Mailgun" },
		{ Icon: Search, label: "VirusTotal" },
		{ Icon: Code2, label: "GitHub" },
	];

	const avatars = [
		{ initials: "JS", tone: "from-rose-400 to-amber-400" },
		{ initials: "MA", tone: "from-cyan-400 to-violet-400" },
		{ initials: "EK", tone: "from-emerald-400 to-cyan-400" },
		{ initials: "TR", tone: "from-violet-400 to-pink-400" },
		{ initials: "AL", tone: "from-amber-400 to-rose-400" },
		{ initials: "+", tone: "from-white/30 to-white/10" },
	];

	return (
		<div className="group/card relative overflow-hidden rounded-3xl border border-white/8 bg-white/2 p-7 transition-all duration-300 hover:border-white/15 hover:bg-white/4">
			{/* Top row: integrations + big number */}
			<div className="flex items-start justify-between gap-4">
				{/* Integrations strip */}
				<div className="flex items-center gap-2 rounded-2xl border border-white/10 bg-black/30 p-2">
					{integrations.map(({ Icon, label }, i) => (
						<motion.div
							key={label}
							initial={{ opacity: 0, scale: 0.7 }}
							whileInView={{ opacity: 1, scale: 1 }}
							viewport={{ once: true }}
							transition={{ delay: i * 0.05, duration: 0.3 }}
							whileHover={{ y: -2 }}
							className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/8 bg-white/4 transition-colors hover:border-white/20"
							title={label}
						>
							<Icon className="h-4 w-4 text-white/70" strokeWidth={1.6} />
						</motion.div>
					))}
				</div>

				{/* Big number */}
				<div className="text-right">
					<div className="font-display text-3xl md:text-4xl font-semibold leading-none text-white">
						542,000
					</div>
					<div className="mt-1 font-mono text-[10px] uppercase tracking-[0.18em] text-white/35">
						Analysts onboarded
					</div>
				</div>
			</div>

			{/* Avatars row */}
			<div className="mt-7 flex -space-x-2">
				{avatars.map((a, i) => (
					<motion.div
						key={i}
						initial={{ opacity: 0, x: -10 }}
						whileInView={{ opacity: 1, x: 0 }}
						viewport={{ once: true }}
						transition={{ delay: 0.3 + i * 0.05 }}
						className={`relative flex h-10 w-10 items-center justify-center rounded-full border-2 border-black bg-linear-to-br ${a.tone} text-[11px] font-semibold text-white/95 ring-0 transition-all duration-200 hover:z-10 hover:scale-110`}
					>
						{a.initials}
					</motion.div>
				))}
			</div>

			{/* Footer text */}
			<div className="mt-8 border-t border-white/6 pt-6">
				<h3 className="font-display text-2xl font-semibold leading-tight text-white">
					Major
					<br />
					User Adoption
				</h3>
				<p className="mt-3 max-w-md text-sm leading-relaxed text-white/45">
					Join a community of 500,000+ analysts and incident-response teams who
					trust DeepMail to triage their inboxes — at scale, every day.
				</p>
			</div>
		</div>
	);
}

// ============================================================
// RIGHT CARD — People love us / Testimonial
// ============================================================
function TestimonialCard() {
	return (
		<div className="group/card relative overflow-hidden rounded-3xl border border-white/8 bg-white/2 p-7 transition-all duration-300 hover:border-white/15 hover:bg-white/4">
			{/* Header */}
			<div>
				<h3 className="font-display text-3xl md:text-4xl font-semibold leading-[1.05] text-white">
					People
					<br />
					love us
				</h3>
				<p className="mt-4 max-w-md text-sm leading-relaxed text-white/45">
					See what analysts are saying about our pipeline and why they keep
					coming back to DeepMail every shift — quaerendi consequat elementum.
				</p>
			</div>

			{/* Floating testimonial card */}
			<div className="relative mt-10">
				{/* Subtle background card (peeking) */}
				<div className="absolute -right-4 -top-4 hidden h-full w-full rounded-2xl border border-white/5 bg-white/2 md:block" />

				<motion.div
					initial={{ opacity: 0, y: 16 }}
					whileInView={{ opacity: 1, y: 0 }}
					viewport={{ once: true }}
					transition={{ duration: 0.45, ease: "easeOut" }}
					whileHover={{ y: -2 }}
					className="relative rounded-2xl border border-white/10 bg-black/40 p-5 shadow-[0_24px_60px_rgba(0,0,0,0.45)] backdrop-blur-sm"
				>
					{/* Avatar + name */}
					<div className="flex items-center gap-3">
						<div className="flex h-9 w-9 items-center justify-center rounded-full bg-linear-to-br from-emerald-400 to-cyan-400 text-[12px] font-semibold text-black">
							TD
						</div>
						<div>
							<p className="text-sm font-medium text-white/90">Tyler Durden</p>
							<p className="text-[11px] text-white/35">Senior IR analyst</p>
						</div>
					</div>

					{/* Quote */}
					<p className="mt-4 text-sm leading-relaxed text-white/60">
						The first rule of{" "}
						<span className="rounded-md border border-emerald-400/40 bg-emerald-400/10 px-1.5 py-0.5 font-mono text-[12px] font-semibold text-emerald-300">
							DeepMail
						</span>{" "}
						is that you triage faster than your queue grows. The second rule of{" "}
						<span className="rounded-md border border-emerald-400/40 bg-emerald-400/10 px-1.5 py-0.5 font-mono text-[12px] font-semibold text-emerald-300">
							DeepMail
						</span>{" "}
						is — well, you get the idea. Ship it.
					</p>
				</motion.div>
			</div>
		</div>
	);
}
