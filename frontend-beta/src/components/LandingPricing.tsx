"use client";
import React, { useState } from "react";
import { LiquidButton } from "./ui/liquid-glass-button";

export function LandingPricing({
	onUploadClick,
	onContactClick,
	onToast,
}: {
	onUploadClick: () => void;
	onContactClick: () => void;
	onToast: (msg: string) => void;
}) {
	const [trialLoading, setTrialLoading] = useState(false);

	const handleTrial = () => {
		setTrialLoading(true);
		setTimeout(() => {
			setTrialLoading(false);
			onToast("Trial Started");
		}, 1500);
	};

	const tiers = [
		{
			name: "Analyst",
			price: "Free",
			desc: "For individual security researchers",
			popular: false,
			features: [
				"10 uploads/day",
				"IOC extraction",
				"Basic scoring",
				"REST API",
			],
			action: (
				<button
					onClick={onUploadClick}
					className="w-full py-3 rounded-lg font-medium text-[14px] glass glass-shine text-(--foreground) hover:bg-(--glass-strong) transition-all"
				>
					Get Started Free
				</button>
			),
		},
		{
			name: "Team",
			price: "$49/mo",
			desc: "For security teams",
			popular: true,
			features: [
				"Unlimited uploads",
				"All Analyst features",
				"Geo-intel enrichment",
				"VirusTotal integration",
				"WebSocket notifications",
				"Priority queue",
			],
			action: (
				<LiquidButton
					onClick={handleTrial}
					disabled={trialLoading}
					className="w-full py-3 rounded-lg font-medium text-[14px] flex items-center justify-center gap-2"
				>
					{trialLoading && (
						<div className="w-4 h-4 rounded-full border-2 border-black/20 border-t-black animate-spin" />
					)}
					{!trialLoading ? "Start Free Trial" : "Securing..."}
				</LiquidButton>
			),
		},
		{
			name: "Enterprise",
			price: "Custom",
			desc: "For SOC platforms",
			popular: false,
			features: [
				"Everything in Team",
				"SLA guarantee",
				"Dedicated instance",
				"Custom integrations",
				"Audit logs",
				"SSO",
			],
			action: (
				<button
					onClick={onContactClick}
					className="w-full py-3 rounded-lg font-medium text-[14px] glass glass-shine text-(--foreground) hover:bg-(--glass-strong) transition-all"
				>
					Contact Sales
				</button>
			),
		},
	];

	return (
		<section
			id="pricing"
			className="py-32 px-6 lg:px-10 max-w-[1120px] mx-auto w-full relative z-10"
		>
			<div className="text-center mb-16">
				<p className="text-[11px] font-medium tracking-[0.16em] uppercase text-(--muted) mb-3">
					Pricing
				</p>
				<h2 className="font-display font-semibold text-[clamp(28px,4vw,42px)] text-gradient tracking-[-0.025em] leading-[1.1]">
					Transparent, scalable security.
				</h2>
			</div>

			<div className="grid md:grid-cols-3 gap-6 items-stretch">
				{tiers.map((t, i) => (
					<div
						key={i}
						className={`glass-shine rounded-2xl p-8 border hover:-translate-y-[2px] transition-all hover:shadow-[-0_12px_48px_rgba(0,0,0,0.8)] flex flex-col ${t.popular ? "border-[rgba(255,255,255,0.14)] bg-(--surface-2) shadow-[0_8px_32px_rgba(0,0,0,0.5)] relative" : "border-(--border) bg-(--surface)"}`}
					>
						{t.popular && (
							<span className="absolute -top-3 left-1/2 -translate-x-1/2 bg-(--surface-3) border border-[rgba(255,255,255,0.2)] text-[10px] font-bold uppercase tracking-widest text-(--foreground) px-3 py-1 rounded-full whitespace-nowrap">
								Most Popular
							</span>
						)}

						<div className="font-display font-semibold text-[13px] text-(--muted) uppercase tracking-wide mb-3">
							{t.name}
						</div>
						<div className="font-display font-bold text-[42px] text-gradient leading-none">
							{t.price}
						</div>
						<div className="font-body font-light text-[13.5px] text-(--muted) mt-2 mb-6">
							{t.desc}
						</div>

						<div className="h-px w-full bg-(--border) my-6" />

						<div className="flex-1 flex flex-col gap-3 mb-8">
							{t.features.map((f, j) => (
								<div key={j} className="flex items-start gap-3">
									<span className="text-(--secondary) mt-0.5">
										✓
									</span>
									<span className="font-body font-light text-[13.5px] text-(--muted)">
										{f}
									</span>
								</div>
							))}
						</div>

						<div className="mt-auto">{t.action}</div>
					</div>
				))}
			</div>
		</section>
	);
}
