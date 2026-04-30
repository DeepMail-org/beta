"use client";

import { useRef, useState } from "react";
import { DotLottieReact, type DotLottie } from "@lottiefiles/dotlottie-react";
import { ArrowUpRight } from "lucide-react";

type Props = {
	label?: string;
	onClick?: () => void;
	disabled?: boolean;
	loading?: boolean;
	loadingLabel?: string;
	type?: "button" | "submit";
	className?: string;
};

export function LottieBookCallButton({
	label = "Book a call",
	onClick,
	disabled,
	loading,
	loadingLabel = "Sending…",
	type = "button",
	className = "",
}: Props) {
	const lottieRef = useRef<DotLottie | null>(null);
	const [hovered, setHovered] = useState(false);

	const handleEnter = () => {
		setHovered(true);
		lottieRef.current?.play();
	};
	const handleLeave = () => {
		setHovered(false);
		lottieRef.current?.pause();
	};

	return (
		<button
			type={type}
			onClick={onClick}
			disabled={disabled || loading}
			onMouseEnter={handleEnter}
			onMouseLeave={handleLeave}
			onFocus={handleEnter}
			onBlur={handleLeave}
			className={
				"group relative inline-flex h-12 items-center justify-center gap-3 overflow-hidden rounded-full liquid-glass border border-white/20 pl-3 pr-6 text-sm font-medium text-white transition-all duration-300 hover:border-white/35 hover:bg-white/8 hover:shadow-[0_0_32px_rgba(255,255,255,0.16)] active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-60 " +
				className
			}
		>
			<span className="relative flex h-9 w-9 items-center justify-center rounded-full border border-white/15 bg-white/5 transition-colors duration-300 group-hover:border-white/30 group-hover:bg-white/8">
				<span
					className="pointer-events-none absolute inset-0 rounded-full opacity-60"
					style={{
						background:
							"radial-gradient(circle, rgba(255,255,255,0.12) 0%, transparent 70%)",
					}}
				/>
				<DotLottieReact
					src="/Book a call.lottie"
					autoplay={false}
					loop
					dotLottieRefCallback={(d) => {
						lottieRef.current = d;
					}}
					style={{
						width: 28,
						height: 28,
						filter: hovered
							? "invert(1) brightness(2)"
							: "invert(1) brightness(1.3) opacity(0.85)",
						transition: "filter 0.3s",
					}}
				/>
			</span>

			<span className="flex items-center gap-2">
				{loading ? (
					<>
						<svg
							className="h-3.5 w-3.5 animate-spin"
							viewBox="0 0 24 24"
							fill="none"
						>
							<circle
								className="opacity-25"
								cx="12"
								cy="12"
								r="10"
								stroke="white"
								strokeWidth="3"
							/>
							<path
								className="opacity-75"
								fill="white"
								d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
							/>
						</svg>
						{loadingLabel}
					</>
				) : (
					<>
						{label}
						<ArrowUpRight className="h-4 w-4 transition-transform duration-300 group-hover:-translate-y-0.5 group-hover:translate-x-0.5" />
					</>
				)}
			</span>
		</button>
	);
}
