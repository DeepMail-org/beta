"use client";
import { Button } from "@/components/ui/button";
import { ArrowUpRight } from "lucide-react";
import { motion, Variants } from "framer-motion";

const containerVariants: Variants = {
	hidden: { opacity: 0 },
	show: {
		opacity: 1,
		transition: { staggerChildren: 0.15 },
	},
};

const itemVariants: Variants = {
	hidden: { opacity: 0, y: 20 },
	show: { opacity: 1, y: 0, transition: { duration: 0.6, ease: "easeOut" } },
};

export function LandingHero({ onUploadClick }: { onUploadClick: () => void }) {
	return (
		<section className="relative min-h-screen flex flex-col pt-32 pb-10 overflow-hidden bg-black">
			{/* Light-streak background — pure CSS, zero SVG filter issues */}
			<div className="absolute inset-0 z-0 bg-black overflow-hidden" aria-hidden>
				{/* Primary streak — bright diagonal line with glow halo */}
				<div style={{
					position: "absolute",
					width: "220%",
					height: "1px",
					left: "-60%",
					top: "49%",
					transform: "rotate(-15.45deg)",
					transformOrigin: "center center",
					background: "rgba(255,255,255,0.85)",
					boxShadow: "0 0 55px 16px rgba(255,255,255,0.18), 0 0 18px 4px rgba(255,255,255,0.32)",
				}} />
				{/* Secondary streak — softer */}
				<div style={{
					position: "absolute",
					width: "220%",
					height: "1px",
					left: "-60%",
					top: "65%",
					transform: "rotate(-15.45deg)",
					transformOrigin: "center center",
					background: "rgba(255,255,255,0.55)",
					boxShadow: "0 0 38px 10px rgba(255,255,255,0.11), 0 0 12px 2px rgba(255,255,255,0.20)",
				}} />
				{/* Tertiary ghost streak */}
				<div style={{
					position: "absolute",
					width: "220%",
					height: "1px",
					left: "-60%",
					top: "75%",
					transform: "rotate(-15.45deg)",
					transformOrigin: "center center",
					background: "rgba(255,255,255,0.28)",
					boxShadow: "0 0 22px 6px rgba(255,255,255,0.06), 0 0 7px 1px rgba(255,255,255,0.10)",
				}} />
			</div>

			{/* Center Content */}
			<motion.div
				className="flex-1 flex flex-col items-center justify-center text-center px-4 relative z-10 w-full max-w-5xl mx-auto mt-10"
				variants={containerVariants}
				initial="hidden"
				animate="show"
			>
				<motion.h1
					variants={itemVariants}
					className="font-display font-normal text-[clamp(80px,18vw,220px)] leading-[1.02] tracking-[-0.024em] mb-2 flex"
				>
					<span className="text-(--foreground)">DeepM</span>
					<span
						className="bg-clip-text text-transparent"
						style={{
							backgroundImage:
								"linear-gradient(to bottom, #ffffff 0%, #e5e7eb 30%, #ffffff 50%, #9ca3af 70%, #4b5563 100%)",
							filter: "drop-shadow(0 0 12px rgba(255,255,255,0.25))",
						}}
					>
						ai
					</span>
					<span className="text-(--foreground)">l</span>
				</motion.h1>

				<motion.p
					variants={itemVariants}
					className="text-[18px] text-[#D1D5DB] font-body opacity-80 leading-8 max-w-md mt-4 py-5"
				>
					The most powerful Email Threat
					<br />
					Intelligence Engine.
				</motion.p>

				<motion.div
					variants={itemVariants}
					className="mt-6.25 flex items-center justify-center"
				>
					<ButtonWithIcon onConsultClick={onUploadClick} />
				</motion.div>
			</motion.div>
		</section>
	);
}

function ButtonWithIcon({ onConsultClick }: { onConsultClick: () => void }) {
	return (
		<Button
			onClick={onConsultClick}
			className="relative text-[15px] font-medium rounded-full h-14 p-1 ps-8 pe-16 group transition-all duration-500 hover:ps-16 hover:pe-8 w-fit overflow-hidden cursor-pointer liquid-glass text-white border border-white/20 shadow-[0_0_20px_rgba(255,255,255,0.1)]"
		>
			<span className="relative z-10 transition-all duration-500 group-hover:tracking-wider">
				Schedule a Consult
			</span>
			<div className="absolute right-1 w-12 h-12 bg-white text-black rounded-full flex items-center justify-center transition-all duration-500 group-hover:right-[calc(100%-52px)] group-hover:rotate-45">
				<ArrowUpRight size={20} />
			</div>
		</Button>
	);
}
