"use client";
import React from "react";
import dynamic from "next/dynamic";
import { cn } from "@/lib/utils";
import { motion } from "motion/react";
import { FileText, ShieldAlert, AlertTriangle, Hash, Globe2, Zap } from "lucide-react";

const GlobePulse = dynamic(
	() => import("@/components/ui/cobe-globe-pulse").then((m) => m.GlobePulse),
	{ ssr: false, loading: () => null },
);

export default function FeaturesSectionDemo() {
	const features = [
		{
			title: "Drag-and-drop .eml ingest",
			description:
				"Drop one file or a thousand. Our parser handles RFC-822, MIME, encoded payloads and weird edge-cases the next vendor's tool will choke on.",
			skeleton: <SkeletonOne />,
			className:
				"col-span-1 lg:col-span-4 border-b lg:border-r border-white/6",
		},
		{
			title: "Live IOC enrichment",
			description:
				"IPs, hashes, URLs and domains light up the moment they land. Geo, ASN, reputation — all on the same canvas.",
			skeleton: <SkeletonTwo />,
			className: "border-b col-span-1 lg:col-span-2 border-white/6",
		},
		{
			title: "Watch the pipeline run",
			description:
				"Six stages stream over a WebSocket: PARSE → IOC → GEO → URL → FILES → SCORE. No refresh required.",
			skeleton: <SkeletonThree />,
			className: "col-span-1 lg:col-span-3 lg:border-r border-white/6",
		},
		{
			title: "Globally aware",
			description:
				"Every signal pinned to a country in real time. Spot campaigns the moment they cross a border.",
			skeleton: <SkeletonFour />,
			className: "col-span-1 lg:col-span-3 border-b lg:border-none",
		},
	];
	return (
		<div className="relative z-20 mx-auto max-w-7xl py-10 lg:py-32">
			<div className="px-8">
				<h4 className="mx-auto max-w-5xl text-center font-display text-3xl font-medium tracking-tight text-white lg:text-5xl lg:leading-tight">
					Built for analysts who don&apos;t have time to wait
				</h4>

				<p className="mx-auto my-4 max-w-2xl text-center text-sm font-normal text-white/40 lg:text-base">
					From raw .eml to a 0–100 threat verdict in seconds. Every layer is a
					component you can audit, swap or query directly.
				</p>
			</div>

			<div className="relative">
				<div className="mt-12 grid grid-cols-1 rounded-2xl lg:grid-cols-6 xl:border border-white/6 bg-white/1.5">
					{features.map((feature) => (
						<FeatureCard key={feature.title} className={feature.className}>
							<FeatureTitle>{feature.title}</FeatureTitle>
							<FeatureDescription>{feature.description}</FeatureDescription>
							<div className="h-full w-full">{feature.skeleton}</div>
						</FeatureCard>
					))}
				</div>
			</div>
		</div>
	);
}

const FeatureCard = ({
	children,
	className,
}: {
	children?: React.ReactNode;
	className?: string;
}) => {
	return (
		<div className={cn(`relative overflow-hidden p-4 sm:p-8`, className)}>
			{children}
		</div>
	);
};

const FeatureTitle = ({ children }: { children?: React.ReactNode }) => {
	return (
		<p className="mx-auto max-w-5xl text-left font-display text-xl tracking-tight text-white md:text-2xl md:leading-snug">
			{children}
		</p>
	);
};

const FeatureDescription = ({ children }: { children?: React.ReactNode }) => {
	return (
		<p
			className={cn(
				"mx-0 my-2 max-w-sm text-left text-sm font-normal text-white/40 md:text-sm",
			)}
		>
			{children}
		</p>
	);
};

// 1) DRAG-AND-DROP TILE
const SkeletonOne = () => {
	return (
		<div className="relative flex h-full gap-10 px-2 py-8">
			<div className="group mx-auto h-full w-full rounded-xl border border-white/8 bg-black/40 p-5 shadow-2xl">
				<div className="flex h-full w-full flex-1 flex-col gap-2">
					{["incoming-001.eml", "phish-attempt.eml", "invoice-fraud.eml", "spear-marketing.eml"].map(
						(name, i) => (
							<motion.div
								key={name}
								initial={{ opacity: 0, x: -20 }}
								whileInView={{ opacity: 1, x: 0 }}
								transition={{ delay: i * 0.08 }}
								viewport={{ once: true }}
								className="flex items-center gap-3 rounded-lg border border-white/5 bg-white/2 p-3"
							>
								<div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-white/5">
									<FileText className="h-4 w-4 text-white/45" />
								</div>
								<div className="flex-1">
									<p className="truncate text-xs font-medium text-white/75">{name}</p>
									<div className="mt-1 flex items-center gap-2">
										<div className="h-1 flex-1 overflow-hidden rounded-full bg-white/4">
											<motion.div
												initial={{ width: 0 }}
												whileInView={{ width: `${75 - i * 12}%` }}
												transition={{ delay: i * 0.1 + 0.3, duration: 0.8 }}
												viewport={{ once: true }}
												className="h-full bg-linear-to-r from-emerald-400/60 to-cyan-400/60"
											/>
										</div>
										<span className="font-mono text-[10px] text-white/30 tabular-nums">
											{(75 - i * 12).toFixed(0)}%
										</span>
									</div>
								</div>
							</motion.div>
						),
					)}
				</div>
			</div>

			<div className="pointer-events-none absolute inset-x-0 bottom-0 z-40 h-40 w-full bg-linear-to-t from-black via-black/60 to-transparent" />
		</div>
	);
};

// 2) IOC ENRICHMENT TILE
const SkeletonTwo = () => {
	const iocs = [
		{ icon: <Globe2 className="h-3 w-3" />, label: "185.220.101.4", tag: "TOR" },
		{ icon: <Hash className="h-3 w-3" />, label: "a1b2c3d4...", tag: "MAL" },
		{ icon: <ShieldAlert className="h-3 w-3" />, label: "evil-co.ru", tag: "C2" },
		{ icon: <AlertTriangle className="h-3 w-3" />, label: "phish.link", tag: "URL" },
		{ icon: <Globe2 className="h-3 w-3" />, label: "10.34.22.1", tag: "PRIV" },
	];
	return (
		<div className="relative flex h-full flex-col items-start gap-3 overflow-hidden p-4">
			<div className="flex w-full flex-wrap gap-2">
				{iocs.map((ioc, idx) => (
					<motion.div
						key={ioc.label}
						initial={{ opacity: 0, scale: 0.8 }}
						whileInView={{ opacity: 1, scale: 1 }}
						transition={{ delay: idx * 0.08 }}
						viewport={{ once: true }}
						whileHover={{ scale: 1.05 }}
						className="flex items-center gap-2 rounded-full border border-white/10 bg-white/3 px-3 py-1.5"
					>
						<span className="text-white/40">{ioc.icon}</span>
						<span className="font-mono text-[11px] text-white/70">{ioc.label}</span>
						<span className="rounded-full bg-red-500/15 px-1.5 py-0.5 font-mono text-[9px] font-semibold text-red-300">
							{ioc.tag}
						</span>
					</motion.div>
				))}
			</div>
			<div className="pointer-events-none absolute inset-y-0 right-0 z-10 h-full w-12 bg-linear-to-l from-black to-transparent" />
		</div>
	);
};

// 3) PIPELINE TILE
const SkeletonThree = () => {
	const stages = ["PARSE", "IOC", "GEO", "URL", "FILES", "SCORE"];
	return (
		<div className="group/pipe relative flex h-full flex-col items-stretch justify-center gap-3 overflow-hidden px-2 py-6">
			{stages.map((stage, i) => (
				<motion.div
					key={stage}
					initial={{ opacity: 0, x: -20 }}
					whileInView={{ opacity: 1, x: 0 }}
					transition={{ delay: i * 0.1 }}
					viewport={{ once: true }}
					className="flex items-center gap-3"
				>
					<span className="w-16 font-mono text-[10px] tabular-nums text-white/35">
						{String(i + 1).padStart(2, "0")}
					</span>
					<div className="flex-1 overflow-hidden rounded-full bg-white/3">
						<motion.div
							initial={{ width: 0 }}
							whileInView={{ width: "100%" }}
							transition={{ delay: i * 0.15 + 0.2, duration: 0.8 }}
							viewport={{ once: true }}
							className="h-1.5 bg-linear-to-r from-white/30 via-white/60 to-white/20"
						/>
					</div>
					<span className="w-12 font-mono text-[10px] tabular-nums text-white/55">
						{stage}
					</span>
				</motion.div>
			))}
		</div>
	);
};

// 4) GLOBE TILE — peek-from-corner globe in project-theme white/glass
const SkeletonFour = () => {
	return (
		<div className="relative h-72 w-full overflow-hidden md:h-80">
			{/* Top fade so the bento title stays readable */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-x-0 top-0 z-10 h-16 bg-linear-to-b from-black/80 to-transparent"
			/>
			{/* Left fade to blend globe into the card edge */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-y-0 left-0 z-10 w-20 bg-linear-to-r from-black/60 to-transparent"
			/>
			{/* Bottom-right fade for natural vignette */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-24 bg-linear-to-t from-black/90 via-black/40 to-transparent"
			/>
			{/* Globe — centered in tile */}
			<div
				className="pointer-events-none absolute inset-0 flex items-center justify-center"
				aria-hidden
			>
				<div className="h-[110%] w-[110%]">
				<GlobePulse
					markers={[
						{ id: "lon", location: [51.51, -0.13], delay: 0 },
						{ id: "nyc", location: [40.71, -74.01], delay: 0.4 },
						{ id: "tok", location: [35.68, 139.65], delay: 0.8 },
						{ id: "del", location: [28.61, 77.21], delay: 1.2 },
						{ id: "syd", location: [-33.87, 151.21], delay: 1.6 },
					]}
					speed={0.004}
					baseColor={[0.52, 0.52, 0.56]}
					markerColor={[1, 1, 1]}
					glowColor={[0.75, 0.75, 0.8]}
					mapBrightness={10}
					interactive={false}
				/>
				</div>
			</div>
		</div>
	);
};

export { Zap };
