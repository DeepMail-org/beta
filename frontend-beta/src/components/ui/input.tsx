"use client";
import * as React from "react";
import { cn } from "@/lib/utils";
import { useMotionTemplate, useMotionValue, motion } from "motion/react";

export type InputProps = React.InputHTMLAttributes<HTMLInputElement>;

const Input = React.forwardRef<HTMLInputElement, InputProps>(
	({ className, type, ...props }, ref) => {
		const radius = 120;
		const [visible, setVisible] = React.useState(false);

		const mouseX = useMotionValue(0);
		const mouseY = useMotionValue(0);

		const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
			const { left, top } = e.currentTarget.getBoundingClientRect();
			mouseX.set(e.clientX - left);
			mouseY.set(e.clientY - top);
		};

		return (
			<motion.div
				style={{
					background: useMotionTemplate`
						radial-gradient(
							${visible ? radius + "px" : "0px"} circle at ${mouseX}px ${mouseY}px,
							rgba(255,255,255,0.35),
							transparent 80%
						)
					`,
				}}
				onMouseMove={handleMouseMove}
				onMouseEnter={() => setVisible(true)}
				onMouseLeave={() => setVisible(false)}
				className="group/input rounded-2xl p-px transition duration-300"
			>
				<input
					type={type}
					className={cn(
						"flex h-11 w-full rounded-2xl border border-white/10 bg-white/2 px-4 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200 hover:border-white/20 focus-visible:border-white/30 focus-visible:bg-white/5 focus-visible:ring-2 focus-visible:ring-white/10 disabled:cursor-not-allowed disabled:opacity-50",
						className,
					)}
					ref={ref}
					{...props}
				/>
			</motion.div>
		);
	},
);
Input.displayName = "Input";

export { Input };
