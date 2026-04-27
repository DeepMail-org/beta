"use client";
import React, { useState, useEffect } from "react";
import { Menu, X } from "lucide-react";
import { LiquidButton } from "./ui/liquid-glass-button";

export function LandingNavbar({
	onUploadClick,
}: {
	onUploadClick: () => void;
}) {
	const [scrolled, setScrolled] = useState(false);
	const [mobileOpen, setMobileOpen] = useState(false);

	useEffect(() => {
		const handleScroll = () => {
			setScrolled(window.scrollY > 60);
		};
		window.addEventListener("scroll", handleScroll, { passive: true });
		return () => window.removeEventListener("scroll", handleScroll);
	}, []);

	return (
		<>
			<nav
				className={`fixed top-0 inset-x-0 z-50 transition-all duration-300 py-5 px-8 flex items-center justify-between ${scrolled ? "bg-black/80 backdrop-blur-xl" : "bg-transparent"}`}
			>
				{/* Left: Logo */}
				<div
					className="flex items-center gap-3 cursor-pointer"
					onClick={() =>
						window.scrollTo({ top: 0, behavior: "smooth" })
					}
				>
					<div className="w-8 h-8 rounded-lg glass-strong flex items-center justify-center shrink-0 border border-(--border)">
						<span className="font-display font-bold text-white text-[15px]">
							D
						</span>
					</div>
					<span className="font-display font-semibold text-[18px]">
						DeepMail
					</span>
				</div>

				{/* Right: CTA */}
				<div className="hidden lg:flex items-center gap-3">
					<button className="px-6 py-2.5 rounded-full text-[14px] font-medium transition-colors liquid-glass text-white border border-white/20 shadow-[0_0_20px_rgba(255,255,255,0.1)] hover:bg-white/10">
						Log In
					</button>
					<button
						onClick={onUploadClick}
						className="px-6 py-2.5 rounded-full text-[14px] font-medium transition-colors liquid-glass text-white border border-white/20 shadow-[0_0_20px_rgba(255,255,255,0.1)] hover:bg-white/10"
					>
						Sign Up
					</button>
				</div>

				{/* Mobile Toggle */}
				<button
					className="lg:hidden p-2 text-(--secondary)"
					onClick={() => setMobileOpen(!mobileOpen)}
				>
					{mobileOpen ? <X /> : <Menu />}
				</button>
			</nav>

			{/* Divider Line */}
			<div
				className={`fixed top-[73px] mt-[3px] inset-x-0 z-40 h-[1px] bg-linear-to-r from-transparent via-(--foreground)/20 to-transparent transition-opacity duration-300 ${scrolled ? "opacity-100" : "opacity-0"}`}
			/>

			{/* Mobile Menu */}
			{mobileOpen && (
				<div className="fixed inset-0 top-[76px] z-40 glass-strong p-6 flex flex-col gap-4 animate-in fade-in slide-in-from-top-4 lg:hidden">
					<div className="mt-8 flex flex-col gap-4">
						<button className="w-full py-3 rounded-full text-center font-medium liquid-glass text-white border border-white/20 shadow-[0_0_20px_rgba(255,255,255,0.1)]">
							Log In
						</button>
						<button
							onClick={() => {
								setMobileOpen(false);
								onUploadClick();
							}}
							className="w-full py-3 rounded-full text-center font-medium liquid-glass text-white border border-white/20 shadow-[0_0_20px_rgba(255,255,255,0.1)]"
						>
							Sign Up
						</button>
					</div>
				</div>
			)}
		</>
	);
}
