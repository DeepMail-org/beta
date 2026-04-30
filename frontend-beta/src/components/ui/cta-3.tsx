"use client";
import { ArrowRightIcon, PlusIcon } from "lucide-react";
import Link from "next/link";

export function CallToAction() {
	return (
		<div className="relative mx-auto flex w-full max-w-3xl flex-col justify-between gap-y-6 border-y border-white/8 bg-[radial-gradient(35%_80%_at_25%_0%,color-mix(in_oklch,white_8%,transparent),transparent)] px-4 py-14">
			{/* Corner plus marks */}
			<PlusIcon className="absolute -left-[11.5px] -top-[12.5px] z-10 size-6 text-white/25" strokeWidth={1} />
			<PlusIcon className="absolute -right-[11.5px] -top-[12.5px] z-10 size-6 text-white/25" strokeWidth={1} />
			<PlusIcon className="absolute -bottom-[12.5px] -left-[11.5px] z-10 size-6 text-white/25" strokeWidth={1} />
			<PlusIcon className="absolute -bottom-[12.5px] -right-[11.5px] z-10 size-6 text-white/25" strokeWidth={1} />

			{/* Side rail lines */}
			<div className="pointer-events-none absolute -inset-y-6 left-0 w-px border-l border-white/6" />
			<div className="pointer-events-none absolute -inset-y-6 right-0 w-px border-r border-white/6" />

			{/* Centre dashed divider */}
			<div className="-z-10 absolute bottom-0 left-1/2 top-0 border-l border-dashed border-white/8" />

			<div className="space-y-2">
				<h2 className="text-center font-display text-2xl font-semibold tracking-tight text-white md:text-3xl">
					Start analysing threats today.
				</h2>
				<p className="text-center text-sm text-white/40">
					Drop your first .eml and get a verdict in seconds. No credit card required.
				</p>
			</div>

			<div className="flex items-center justify-center gap-3">
				<Link
					href="/contact"
					className="inline-flex h-10 items-center gap-2 rounded-full border border-white/12 bg-white/4 px-5 text-sm font-medium text-white/80 transition-all duration-200 hover:border-white/25 hover:bg-white/8 hover:text-white"
				>
					Contact Sales
				</Link>
				<Link
					href="/signup"
					className="inline-flex h-10 items-center gap-2 rounded-full border border-white/20 bg-white/8 px-5 text-sm font-medium text-white transition-all duration-200 hover:border-white/35 hover:bg-white/12 hover:shadow-[0_0_24px_rgba(255,255,255,0.12)]"
				>
					Get Started <ArrowRightIcon className="ml-0.5 size-4" />
				</Link>
			</div>
		</div>
	);
}
