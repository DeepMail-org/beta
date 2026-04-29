"use client";
import { useState } from "react";
import { Menu, X } from "lucide-react";
import Link from "next/link";

export function LandingNavbar({
	onUploadClick,
}: {
	onUploadClick: () => void;
}) {
	const [mobileOpen, setMobileOpen] = useState(false);

	return (
		<>
			<nav
				className="fixed top-0 inset-x-0 z-50 py-3 px-6 flex items-center justify-between bg-transparent"
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

				{/* Right: nav links + CTA */}
				<div className="hidden lg:flex items-center gap-6">
					<Link
						href="/contact"
						className="text-[14px] font-medium text-white/50 hover:text-white/90 transition-colors duration-200"
					>
						Contact
					</Link>
					<div className="flex items-center gap-2.5">
						<Link
							href="/login"
							className="px-5 py-2.5 rounded-full text-[14px] font-medium transition-all duration-200 liquid-glass text-white border border-white/18 hover:bg-white/8 hover:border-white/32 hover:shadow-[0_0_24px_rgba(255,255,255,0.12)]"
						>
							Log In
						</Link>
						<Link
							href="/signup"
							className="px-5 py-2.5 rounded-full text-[14px] font-medium transition-all duration-200 liquid-glass text-white border border-white/18 hover:bg-white/8 hover:border-white/32 hover:shadow-[0_0_24px_rgba(255,255,255,0.12)]"
						>
							Sign Up
						</Link>
					</div>
				</div>

				{/* Mobile Toggle */}
				<button
					className="lg:hidden p-2 text-(--secondary)"
					onClick={() => setMobileOpen(!mobileOpen)}
				>
					{mobileOpen ? <X /> : <Menu />}
				</button>
			</nav>


			{/* Mobile Menu */}
			{mobileOpen && (
				<div className="fixed inset-0 top-19 z-40 glass-strong p-6 flex flex-col gap-4 animate-in fade-in slide-in-from-top-4 lg:hidden">
					<div className="mt-4 flex flex-col gap-3">
						<Link
							href="/contact"
							onClick={() => setMobileOpen(false)}
							className="w-full py-3 rounded-full text-center font-medium text-white/60 border border-white/10 hover:text-white hover:border-white/20 transition-all duration-200"
						>
							Contact
						</Link>
						<Link
							href="/login"
							onClick={() => setMobileOpen(false)}
							className="w-full py-3 rounded-full text-center font-medium liquid-glass text-white border border-white/18 hover:bg-white/8 transition-all duration-200"
						>
							Log In
						</Link>
						<Link
							href="/signup"
							onClick={() => setMobileOpen(false)}
							className="w-full py-3 rounded-full text-center font-medium liquid-glass text-white border border-white/18 hover:bg-white/8 transition-all duration-200"
						>
							Sign Up
						</Link>
					</div>
				</div>
			)}
		</>
	);
}
