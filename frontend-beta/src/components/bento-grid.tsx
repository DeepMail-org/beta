"use client";
import { cn } from "@/lib/utils";
import { BentoGrid, BentoGridItem } from "@/components/ui/bento-grid";
import {
	IconShieldCheck,
	IconBolt,
	IconWorldWww,
	IconHash,
	IconBrandGithub,
} from "@tabler/icons-react";
import { motion } from "motion/react";

export default function BentoGridClass() {
	return (
		<BentoGrid className="mx-auto max-w-5xl px-6 md:auto-rows-[20rem]">
			{items.map((item, i) => (
				<BentoGridItem
					key={i}
					title={item.title}
					description={item.description}
					header={item.header}
					className={cn("[&>p:text-lg]", item.className)}
					icon={item.icon}
				/>
			))}
		</BentoGrid>
	);
}

// 1) IOC TAG SHUFFLE
const SkeletonOne = () => {
	const variants = {
		initial: { x: 0 },
		animate: { x: 10, rotate: 4, transition: { duration: 0.2 } },
	};
	const variantsSecond = {
		initial: { x: 0 },
		animate: { x: -10, rotate: -4, transition: { duration: 0.2 } },
	};

	return (
		<motion.div
			initial="initial"
			whileHover="animate"
			className="flex h-full min-h-24 w-full flex-1 flex-col gap-2"
		>
			<motion.div
				variants={variants}
				className="flex flex-row items-center gap-2 rounded-full border border-white/10 bg-black/40 p-2"
			>
				<div className="h-6 w-6 shrink-0 rounded-full bg-linear-to-r from-rose-500 to-amber-500" />
				<div className="h-3 w-full rounded-full bg-white/8" />
				<span className="font-mono text-[9px] font-semibold tracking-wider text-rose-300">
					CRIT
				</span>
			</motion.div>
			<motion.div
				variants={variantsSecond}
				className="ml-auto flex w-3/4 flex-row items-center gap-2 rounded-full border border-white/10 bg-black/40 p-2"
			>
				<div className="h-3 w-full rounded-full bg-white/8" />
				<div className="h-6 w-6 shrink-0 rounded-full bg-linear-to-r from-cyan-500 to-violet-500" />
			</motion.div>
			<motion.div
				variants={variants}
				className="flex flex-row items-center gap-2 rounded-full border border-white/10 bg-black/40 p-2"
			>
				<div className="h-6 w-6 shrink-0 rounded-full bg-linear-to-r from-emerald-500 to-cyan-500" />
				<div className="h-3 w-full rounded-full bg-white/8" />
				<span className="font-mono text-[9px] font-semibold tracking-wider text-emerald-300">
					SAFE
				</span>
			</motion.div>
		</motion.div>
	);
};

// 2) PIPELINE BARS
const SkeletonTwo = () => {
	const variants = {
		initial: { width: 0 },
		animate: { width: "100%", transition: { duration: 0.2 } },
		hover: { width: ["0%", "100%"], transition: { duration: 2 } },
	};
	const arr = new Array(6).fill(0);
	return (
		<motion.div
			initial="initial"
			animate="animate"
			whileHover="hover"
			className="flex h-full min-h-24 w-full flex-1 flex-col gap-2"
		>
			{arr.map((_, i) => (
				<motion.div
					key={"skeleton-two" + i}
					variants={variants}
					style={{ maxWidth: Math.random() * (100 - 40) + 40 + "%" }}
					className="flex h-3 w-full flex-row items-center rounded-full bg-linear-to-r from-white/20 via-white/35 to-white/10"
				/>
			))}
		</motion.div>
	);
};

// 3) THREAT METER
const SkeletonThree = () => {
	const variants = {
		initial: { backgroundPosition: "0 50%" },
		animate: { backgroundPosition: ["0% 50%", "100% 50%", "0 50%"] },
	};
	return (
		<motion.div
			initial="initial"
			animate="animate"
			variants={variants}
			transition={{
				duration: 5,
				repeat: Infinity,
				repeatType: "reverse",
			}}
			className="relative flex h-full min-h-24 w-full flex-1 items-center justify-center overflow-hidden rounded-lg"
			style={{
				background:
					"linear-gradient(-45deg, #0f1424, #1a1230, #052028, #0a1f1a)",
				backgroundSize: "400% 400%",
			}}
		>
			<div
				className="absolute inset-0 rounded-lg"
				style={{ boxShadow: "inset 0 0 60px rgba(0,0,0,0.6)" }}
			/>
			<div className="relative font-display font-bold text-white/90">
				<span className="font-mono text-5xl tabular-nums">87</span>
				<span className="ml-1 font-mono text-sm text-white/55">
					/100
				</span>
			</div>
		</motion.div>
	);
};

// 4) GEO SHUFFLE CARDS
const SkeletonFour = () => {
	const first = {
		initial: { x: 20, rotate: -5 },
		hover: { x: 0, rotate: 0 },
	};
	const second = {
		initial: { x: -20, rotate: 5 },
		hover: { x: 0, rotate: 0 },
	};
	const card =
		"flex h-full w-1/3 flex-col items-center justify-center rounded-2xl border border-white/8 bg-black/50 p-4";
	return (
		<motion.div
			initial="initial"
			animate="animate"
			whileHover="hover"
			className="flex h-full min-h-24 w-full flex-1 flex-row gap-2"
		>
			<motion.div variants={first} className={card}>
				<IconWorldWww className="h-8 w-8 text-rose-400" stroke={1.4} />
				<p className="mt-3 text-center text-xs font-semibold text-white/70">
					RU · 185.220.x
				</p>
				<p className="mt-2 rounded-full border border-rose-500/40 bg-rose-500/10 px-2 py-0.5 text-[10px] text-rose-300">
					Hostile
				</p>
			</motion.div>
			<motion.div className="relative z-20 flex h-full w-1/3 flex-col items-center justify-center rounded-2xl border border-white/15 bg-black/70 p-4">
				<IconShieldCheck
					className="h-8 w-8 text-emerald-400"
					stroke={1.4}
				/>
				<p className="mt-3 text-center text-xs font-semibold text-white/70">
					US · CDN edge
				</p>
				<p className="mt-2 rounded-full border border-emerald-500/40 bg-emerald-500/10 px-2 py-0.5 text-[10px] text-emerald-300">
					Trusted
				</p>
			</motion.div>
			<motion.div variants={second} className={card}>
				<IconBolt className="h-8 w-8 text-amber-400" stroke={1.4} />
				<p className="mt-3 text-center text-xs font-semibold text-white/70">
					CN · 39.x.x.x
				</p>
				<p className="mt-2 rounded-full border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-[10px] text-amber-300">
					Watchlist
				</p>
			</motion.div>
		</motion.div>
	);
};

// 5) ANALYST CHAT
const SkeletonFive = () => {
	const variants = {
		initial: { x: 0 },
		animate: { x: 10, rotate: 4, transition: { duration: 0.2 } },
	};
	const variantsSecond = {
		initial: { x: 0 },
		animate: { x: -10, rotate: -4, transition: { duration: 0.2 } },
	};

	return (
		<motion.div
			initial="initial"
			whileHover="animate"
			className="flex h-full min-h-24 w-full flex-1 flex-col gap-2"
		>
			<motion.div
				variants={variants}
				className="flex flex-row items-start gap-2 rounded-2xl border border-white/8 bg-black/40 p-2"
			>
				<div className="h-7 w-7 shrink-0 rounded-full bg-linear-to-br from-cyan-500 to-violet-500" />
				<p className="text-[11px] leading-snug text-white/55">
					IOC{" "}
					<span className="font-mono text-white/75">a1b2c3d4</span>{" "}
					matches a known credential-stealer family.
				</p>
			</motion.div>
			<motion.div
				variants={variantsSecond}
				className="ml-auto flex w-3/4 flex-row items-center justify-end gap-2 rounded-full border border-white/8 bg-black/40 p-2"
			>
				<p className="text-[11px] text-white/55">Quarantine, please.</p>
				<div className="h-6 w-6 shrink-0 rounded-full bg-linear-to-r from-emerald-500 to-cyan-500" />
			</motion.div>
		</motion.div>
	);
};

const items = [
	{
		title: "Live IOC tagging",
		description: (
			<span className="text-sm">
				Every artifact gets a verdict the moment it lands.
			</span>
		),
		header: <SkeletonOne />,
		className: "md:col-span-1",
		icon: <IconShieldCheck className="h-4 w-4 text-white/45" />,
	},
	{
		title: "Six-stage pipeline",
		description: (
			<span className="text-sm">
				PARSE · IOC · GEO · URL · FILES · SCORE — streamed end to end.
			</span>
		),
		header: <SkeletonTwo />,
		className: "md:col-span-1",
		icon: <IconBolt className="h-4 w-4 text-white/45" />,
	},
	{
		title: "Threat confidence",
		description: (
			<span className="text-sm">
				One number, six signals. Calibrated for SOCs.
			</span>
		),
		header: <SkeletonThree />,
		className: "md:col-span-1",
		icon: <IconHash className="h-4 w-4 text-white/45" />,
	},
	{
		title: "Geo-aware verdicts",
		description: (
			<span className="text-sm">
				Reputation is regional. We pin every artifact to its origin so
				you can spot the campaign, not just the message.
			</span>
		),
		header: <SkeletonFour />,
		className: "md:col-span-2",
		icon: <IconWorldWww className="h-4 w-4 text-white/45" />,
	},
	{
		title: "Analyst-friendly notes",
		description: (
			<span className="text-sm">
				Plain-English context for every flag.
			</span>
		),
		header: <SkeletonFive />,
		className: "md:col-span-1",
		icon: <IconBrandGithub className="h-4 w-4 text-white/45" />,
	},
];
