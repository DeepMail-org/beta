"use client";

import { useState } from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import {
	Check,
	Shield,
	Zap,
	Sparkles,
	CreditCard,
	Receipt,
	Lock,
	Download,
} from "lucide-react";
import {
	CreditCardForm,
	type CardState,
	type CardValidity,
} from "@/components/ui/credit-card-form";

type Plan = {
	id: "starter" | "pro" | "enterprise";
	name: string;
	priceMonthly: number;
	tagline: string;
	highlights: string[];
	icon: typeof Shield;
	popular?: boolean;
};

const PLANS: Plan[] = [
	{
		id: "starter",
		name: "Starter",
		priceMonthly: 0,
		tagline: "For solo analysts kicking the tires.",
		highlights: [
			"500 .eml scans / month",
			"Basic IOC extraction",
			"Geo-IP enrichment",
			"7-day report retention",
		],
		icon: Shield,
	},
	{
		id: "pro",
		name: "Pro",
		priceMonthly: 79,
		tagline: "Everything an SOC analyst needs, daily.",
		highlights: [
			"50,000 .eml scans / month",
			"Full pipeline (PARSE → SCORE)",
			"VirusTotal & AbuseIPDB lookups",
			"WebSocket streaming + REST API",
			"30-day report retention",
		],
		icon: Zap,
		popular: true,
	},
	{
		id: "enterprise",
		name: "Enterprise",
		priceMonthly: 499,
		tagline: "Custom limits, SSO, dedicated support.",
		highlights: [
			"Unlimited scans",
			"SSO / SAML",
			"Private threat-intel feeds",
			"On-prem worker option",
			"99.95% SLA",
		],
		icon: Sparkles,
	},
];

const INVOICES = [
	{ id: "INV-2026-04-2401", date: "2026-04-01", amount: 79, status: "Paid" },
	{ id: "INV-2026-03-2401", date: "2026-03-01", amount: 79, status: "Paid" },
	{ id: "INV-2026-02-2401", date: "2026-02-01", amount: 79, status: "Paid" },
];

export default function PaymentsPage() {
	const [billing, setBilling] = useState<"monthly" | "yearly">("monthly");
	const [selectedPlan, setSelectedPlan] = useState<Plan["id"]>("pro");
	const [card, setCard] = useState<CardState | null>(null);
	const [saving, setSaving] = useState(false);
	const [saved, setSaved] = useState(false);

	const activePlan = PLANS.find((p) => p.id === selectedPlan)!;
	const yearlyDiscount = 0.18;
	const displayPrice =
		billing === "monthly"
			? activePlan.priceMonthly
			: Math.round(activePlan.priceMonthly * 12 * (1 - yearlyDiscount));

	const handleCardSubmit = async (
		_state: CardState,
		validity: CardValidity,
	) => {
		if (!validity.allValid) return;
		setSaving(true);
		await new Promise((r) => setTimeout(r, 1100));
		setSaving(false);
		setSaved(true);
	};

	return (
		<div className="relative min-h-screen overflow-hidden bg-black text-white">
			{/* Ambient backdrop */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-0"
				style={{
					background:
						"radial-gradient(ellipse 70% 55% at 50% 0%, rgba(255,255,255,0.04) 0%, transparent 70%)",
				}}
			/>
			<div
				aria-hidden
				className="pointer-events-none absolute -left-48 top-1/4 h-120 w-120 rounded-full"
				style={{
					background:
						"radial-gradient(circle, rgba(120,145,255,0.07) 0%, transparent 70%)",
					filter: "blur(50px)",
				}}
			/>
			<div
				aria-hidden
				className="pointer-events-none absolute -right-48 bottom-1/4 h-105 w-105 rounded-full"
				style={{
					background:
						"radial-gradient(circle, rgba(220,90,200,0.05) 0%, transparent 70%)",
					filter: "blur(50px)",
				}}
			/>

			{/* Navbar */}
			<nav className="fixed inset-x-0 top-0 z-50 flex items-center justify-between bg-transparent px-6 py-4 md:px-10">
				<Link href="/" className="group flex items-center gap-2.5">
					<div className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/10 bg-white/5">
						<span className="font-display text-[15px] font-bold text-white">D</span>
					</div>
					<span className="font-display text-[17px] font-semibold text-white/90">
						DeepMail
					</span>
				</Link>
				<Link
					href="/"
					className="flex items-center gap-1.5 text-sm text-white/40 transition-colors duration-200 hover:text-white/80"
				>
					← Back to home
				</Link>
			</nav>

			<main className="relative mx-auto w-full max-w-6xl px-6 pb-20 pt-32 md:px-10 lg:pt-40">
				{/* Header */}
				<motion.header
					initial={{ opacity: 0, y: 24 }}
					animate={{ opacity: 1, y: 0 }}
					transition={{ duration: 0.45 }}
					className="mb-12 flex flex-col items-start gap-4"
				>
					<div className="inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/3 px-3 py-1">
						<CreditCard className="h-3 w-3 text-white/55" strokeWidth={1.5} />
						<span className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/55">
							Billing
						</span>
					</div>
					<h1 className="font-display text-4xl font-semibold tracking-tight text-white md:text-5xl">
						Payments &amp; Subscription
					</h1>
					<p className="max-w-xl text-sm text-white/40 leading-relaxed">
						Pick the plan that fits your team, then add a payment method. You can
						switch plans, cancel or download invoices at any time.
					</p>
				</motion.header>

				{/* Billing toggle */}
				<div className="mb-8 flex items-center gap-3">
					<div className="inline-flex items-center gap-1 rounded-full border border-white/10 bg-white/2 p-1">
						{(["monthly", "yearly"] as const).map((opt) => (
							<button
								key={opt}
								onClick={() => setBilling(opt)}
								className={`rounded-full px-4 py-1.5 text-xs font-medium transition-all duration-200 ${
									billing === opt
										? "bg-white/10 text-white"
										: "text-white/50 hover:text-white/80"
								}`}
							>
								{opt === "monthly" ? "Monthly" : "Yearly"}
							</button>
						))}
					</div>
					{billing === "yearly" && (
						<span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 font-mono text-[10px] font-semibold uppercase tracking-wider text-emerald-300">
							Save 18%
						</span>
					)}
				</div>

				{/* Plans */}
				<section className="mb-16 grid grid-cols-1 gap-4 md:grid-cols-3">
					{PLANS.map((plan) => {
						const Icon = plan.icon;
						const isActive = selectedPlan === plan.id;
						const price =
							billing === "monthly"
								? plan.priceMonthly
								: Math.round(plan.priceMonthly * 12 * (1 - yearlyDiscount));
						return (
							<motion.button
								key={plan.id}
								onClick={() => setSelectedPlan(plan.id)}
								whileHover={{ y: -2 }}
								className={`relative flex flex-col gap-5 rounded-3xl border p-6 text-left transition-all duration-200 ${
									isActive
										? "border-white/25 bg-white/4 shadow-[0_0_32px_rgba(255,255,255,0.08)]"
										: "border-white/8 bg-white/2 hover:border-white/15"
								}`}
							>
								{plan.popular && (
									<span className="absolute right-4 top-4 rounded-full border border-amber-400/40 bg-amber-400/10 px-2 py-0.5 font-mono text-[9px] font-semibold uppercase tracking-wider text-amber-300">
										Most popular
									</span>
								)}
								<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
									<Icon className="h-5 w-5 text-white/70" strokeWidth={1.5} />
								</div>
								<div>
									<h3 className="font-display text-xl font-semibold text-white">
										{plan.name}
									</h3>
									<p className="mt-1 text-xs text-white/40">{plan.tagline}</p>
								</div>
								<div className="flex items-end gap-1">
									<span className="font-display text-3xl font-semibold text-white">
										${price}
									</span>
									<span className="mb-1 text-xs text-white/40">
										/{billing === "monthly" ? "mo" : "yr"}
									</span>
								</div>
								<ul className="space-y-2">
									{plan.highlights.map((h) => (
										<li
											key={h}
											className="flex items-start gap-2 text-xs text-white/60"
										>
											<Check
												className="mt-0.5 h-3.5 w-3.5 shrink-0 text-emerald-400"
												strokeWidth={2}
											/>
											<span>{h}</span>
										</li>
									))}
								</ul>
								{isActive && (
									<motion.div
										layoutId="active-plan"
										className="absolute inset-0 rounded-3xl ring-2 ring-white/20"
										style={{ pointerEvents: "none" }}
									/>
								)}
							</motion.button>
						);
					})}
				</section>

				{/* Order summary + Card form */}
				<section className="mb-16 grid grid-cols-1 gap-8 lg:grid-cols-12">
					{/* Order summary */}
					<aside className="lg:col-span-4 flex flex-col gap-6">
						<div className="rounded-3xl border border-white/8 bg-white/2 p-6">
							<h2 className="font-display text-lg font-semibold text-white">
								Order summary
							</h2>
							<div className="mt-5 flex items-start justify-between border-t border-white/6 pt-5">
								<div>
									<p className="text-sm font-medium text-white/85">
										{activePlan.name} plan
									</p>
									<p className="text-xs text-white/35">
										Billed {billing}
									</p>
								</div>
								<p className="font-mono text-sm tabular-nums text-white/85">
									${displayPrice}
								</p>
							</div>
							<div className="mt-3 flex items-start justify-between text-xs text-white/45">
								<span>Tax (estimated)</span>
								<span className="font-mono tabular-nums">
									${Math.round(displayPrice * 0.18)}
								</span>
							</div>
							<div className="mt-5 flex items-end justify-between border-t border-white/6 pt-5">
								<span className="text-xs uppercase tracking-wider text-white/40">
									Total due today
								</span>
								<span className="font-display text-2xl font-semibold tabular-nums text-white">
									${displayPrice + Math.round(displayPrice * 0.18)}
								</span>
							</div>
						</div>

						<div className="flex items-start gap-3 rounded-2xl border border-white/6 bg-white/[0.015] p-4">
							<Lock className="mt-0.5 h-4 w-4 shrink-0 text-emerald-400/80" strokeWidth={1.5} />
							<p className="text-xs text-white/45 leading-relaxed">
								Card data is tokenised by our PCI-DSS-compliant processor.
								DeepMail never stores raw card details.
							</p>
						</div>
					</aside>

					{/* Credit card form */}
					<div className="lg:col-span-8">
						<div className="rounded-3xl border border-white/8 bg-white/2 p-6 md:p-8">
							{saved ? (
								<div className="flex flex-col items-center justify-center gap-5 py-16">
									<div className="flex h-16 w-16 items-center justify-center rounded-2xl border border-emerald-400/30 bg-emerald-400/10">
										<Check className="h-7 w-7 text-emerald-400" strokeWidth={2} />
									</div>
									<h2 className="font-display text-2xl font-semibold text-white">
										Payment method saved
									</h2>
									<p className="max-w-sm text-center text-sm text-white/45">
										You&apos;re subscribed to the{" "}
										<span className="text-white/80">{activePlan.name}</span>{" "}
										plan. A receipt is on the way to your inbox.
									</p>
									<button
										onClick={() => {
											setSaved(false);
											setCard(null);
										}}
										className="mt-1 text-xs text-white/35 transition-colors hover:text-white/70"
									>
										Use a different card
									</button>
								</div>
							) : (
								<>
									<header className="mb-6">
										<h2 className="font-display text-xl font-semibold text-white">
											Payment method
										</h2>
										<p className="mt-1 text-xs text-white/40">
											Visa, Mastercard, Amex — auto-detected.
										</p>
									</header>

									<CreditCardForm
										onChange={(state) => setCard(state)}
										onSubmit={handleCardSubmit}
										showSubmit
									/>

									{saving && (
										<p className="mt-4 text-center text-xs text-white/40">
											Saving payment method…
										</p>
									)}
								</>
							)}
						</div>
					</div>
				</section>

				{/* Recent invoices */}
				<section>
					<header className="mb-5 flex items-center gap-3">
						<div className="flex h-9 w-9 items-center justify-center rounded-xl border border-white/10 bg-white/5">
							<Receipt className="h-4 w-4 text-white/65" strokeWidth={1.5} />
						</div>
						<div>
							<h2 className="font-display text-lg font-semibold text-white">
								Recent invoices
							</h2>
							<p className="text-xs text-white/40">
								Last three billing cycles
							</p>
						</div>
					</header>
					<div className="overflow-hidden rounded-2xl border border-white/8 bg-white/2">
						<table className="w-full text-left">
							<thead className="bg-white/[0.015] text-[10px] uppercase tracking-[0.18em] text-white/35">
								<tr>
									<th className="px-5 py-3 font-medium">Invoice</th>
									<th className="px-5 py-3 font-medium">Date</th>
									<th className="px-5 py-3 font-medium">Amount</th>
									<th className="px-5 py-3 font-medium">Status</th>
									<th className="px-5 py-3 font-medium" aria-label="Actions" />
								</tr>
							</thead>
							<tbody>
								{INVOICES.map((inv, i) => (
									<tr
										key={inv.id}
										className={`text-sm transition-colors hover:bg-white/[0.015] ${
											i !== 0 ? "border-t border-white/5" : ""
										}`}
									>
										<td className="px-5 py-4 font-mono text-xs text-white/80">
											{inv.id}
										</td>
										<td className="px-5 py-4 text-white/55">{inv.date}</td>
										<td className="px-5 py-4 font-mono tabular-nums text-white/80">
											${inv.amount}.00
										</td>
										<td className="px-5 py-4">
											<span className="rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-[10px] font-semibold uppercase tracking-wider text-emerald-300">
												{inv.status}
											</span>
										</td>
										<td className="px-5 py-4 text-right">
											<button className="inline-flex items-center gap-1.5 text-xs text-white/45 transition-colors hover:text-white">
												<Download className="h-3 w-3" />
												PDF
											</button>
										</td>
									</tr>
								))}
							</tbody>
						</table>
					</div>
				</section>
			</main>
		</div>
	);
}
