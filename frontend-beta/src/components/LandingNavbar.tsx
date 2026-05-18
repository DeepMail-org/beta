"use client";
import { useState } from "react";
import { Menu, X } from "lucide-react";
import Link from "next/link";
import { LiquidButton } from "@/components/ui/liquid-glass-button";

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
					<div className="flex items-center gap-2.5">
						<Link href="/login">
							<LiquidButton size="sm" className="rounded-full px-5 py-2.5 text-[14px] font-medium text-white">
								Log In
							</LiquidButton>
						</Link>
						<Link href="/signup">
							<LiquidButton size="sm" className="rounded-full px-5 py-2.5 text-[14px] font-medium text-white">
								Sign Up
							</LiquidButton>
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
						<Link href="/login" onClick={() => setMobileOpen(false)}>
						<LiquidButton size="default" className="w-full rounded-full text-center font-medium text-white">
							Log In
						</LiquidButton>
					</Link>
					<Link href="/signup" onClick={() => setMobileOpen(false)}>
						<LiquidButton size="default" className="w-full rounded-full text-center font-medium text-white">
							Sign Up
						</LiquidButton>
					</Link>
					</div>
				</div>
			)}
		</>
	);
}
