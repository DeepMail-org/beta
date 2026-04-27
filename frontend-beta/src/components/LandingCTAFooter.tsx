"use client";
import React from "react";
import { LiquidButton } from "./ui/liquid-glass-button";

export function LandingCTA({ onUploadClick }: { onUploadClick: () => void }) {
	return (
		<section className="w-full relative py-36 text-center border-t border-(--border) bg-(--surface) overflow-hidden">
			<div className="absolute inset-0 bg-(--bg-glow) pointer-events-none" />

			<div className="relative z-10 px-6 max-w-2xl mx-auto">
				<p className="text-[11px] font-medium tracking-[0.16em] uppercase text-(--muted) mb-3">
					Architecture
				</p>
				<h2 className="font-display font-semibold text-[clamp(32px,4vw,56px)] text-gradient tracking-[-0.025em] leading-[1.1] mb-4">
					Start analyzing threats today.
				</h2>
				<p className="font-body font-light text-[17px] text-(--muted) mb-10 max-w-[420px] mx-auto">
					Upload your first .eml file and get full IOC analysis in
					seconds.
				</p>
				<LiquidButton
					onClick={onUploadClick}
					className="px-8 py-3.5 rounded-full text-[15px] cursor-pointer inline-flex font-medium"
				>
					Get Started
				</LiquidButton>
			</div>
		</section>
	);
}

export function LandingFooter() {
	return (
		<footer className="w-full bg-(--background) border-t border-(--border)">
			<div className="max-w-[1120px] mx-auto px-6 lg:px-10 py-16 grid grid-cols-2 md:grid-cols-5 gap-10">
				<div className="col-span-2">
					<div className="flex items-center gap-3 mb-4">
						<div className="w-8 h-8 rounded-lg glass-strong flex items-center justify-center shrink-0 border border-(--border)">
							<span className="font-display font-bold text-white text-[15px]">
								D
							</span>
						</div>
						<span className="font-display font-semibold text-[18px]">
							<span className="text-(--foreground)">
								Deep
							</span>
							<span className="text-gradient">Mail</span>
						</span>
					</div>
					<p className="text-[13px] text-(--muted) font-body font-light max-w-sm">
						DeepMail — Email Threat Intelligence. Built for analysts
						who need instant visibility into email headers, URLs,
						and attachment hashes.
					</p>
				</div>

				<div>
					<h4 className="font-display font-medium text-[11px] uppercase tracking-wide text-(--dimmed) mb-4">
						Product
					</h4>
					<ul className="space-y-3">
						{["Features", "Pricing", "Changelog"].map((l) => (
							<li key={l}>
								<a
									href="#"
									className="text-[13px] text-(--muted) hover:text-(--secondary) transition-colors"
								>
									{l}
								</a>
							</li>
						))}
					</ul>
				</div>

				<div>
					<h4 className="font-display font-medium text-[11px] uppercase tracking-wide text-(--dimmed) mb-4">
						Resources
					</h4>
					<ul className="space-y-3">
						{["Docs", "API Reference", "Architecture"].map((l) => (
							<li key={l}>
								<a
									href="#"
									className="text-[13px] text-(--muted) hover:text-(--secondary) transition-colors"
								>
									{l}
								</a>
							</li>
						))}
					</ul>
				</div>

				<div>
					<h4 className="font-display font-medium text-[11px] uppercase tracking-wide text-(--dimmed) mb-4">
						Company
					</h4>
					<ul className="space-y-3">
						{["GitHub", "Security", "Status"].map((l) => (
							<li key={l}>
								<a
									href="#"
									className="text-[13px] text-(--muted) hover:text-(--secondary) transition-colors"
								>
									{l}
								</a>
							</li>
						))}
					</ul>
				</div>
			</div>

			<div className="border-t border-(--border) max-w-[1120px] mx-auto px-6 lg:px-10 py-6 flex flex-col sm:flex-row items-center justify-between gap-4">
				<p className="text-[12px] text-(--dimmed) font-light font-body">
					© 2026 DeepMail. Built for defenders.
				</p>
				<a
					href="#"
					className="text-(--muted) hover:text-(--foreground) transition-colors flex items-center gap-2"
				>
					<span className="text-[12px]">GitHub</span>
					<svg
						className="w-4 h-4"
						fill="currentColor"
						viewBox="0 0 24 24"
					>
						<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
					</svg>
				</a>
			</div>
		</footer>
	);
}
