"use client";

import React from "react";
import { Search, Bell, Menu } from "lucide-react";
import { usePathname } from "next/navigation";

export function TopNav({ onMenuClick }: { onMenuClick?: () => void }) {
	const pathname = usePathname();

	// Very simple breadcrumb logic based on pathname
	const segments = pathname.split("/").filter(Boolean);
	const title = segments.length > 0 
		? segments[segments.length - 1].split("-").map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(" ")
		: "Dashboard";

	return (
		<header className="sticky top-0 z-30 flex items-center justify-between px-4 lg:px-8 h-16 bg-background/80 backdrop-blur-xl border-b border-border">
			<div className="flex items-center gap-4">
				<button 
					onClick={onMenuClick}
					className="md:hidden p-2 -ml-2 text-muted hover:text-white rounded-lg hover:bg-white/5 transition-colors"
				>
					<Menu className="w-5 h-5" />
				</button>
				
				<div className="flex flex-col">
					<div className="text-[11px] font-medium text-muted/70 uppercase tracking-widest hidden sm:block">
						DeepMail Platform
					</div>
					<h1 className="text-lg font-display font-semibold text-white">
						{title}
					</h1>
				</div>
			</div>

			<div className="flex items-center gap-3">
				{/* Global Search */}
				<div className="relative hidden md:block w-64">
					<div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
						<Search className="w-4 h-4 text-muted" />
					</div>
					<input
						type="text"
						className="block w-full pl-9 pr-3 py-2 bg-surface/50 border border-border rounded-lg text-sm placeholder-muted text-white focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent/50 transition-all shadow-inner shadow-black/50"
						placeholder="Search IOCs, cases, rules..."
					/>
					<div className="absolute inset-y-0 right-0 pr-2 flex items-center pointer-events-none">
						<span className="text-[10px] text-muted border border-border/50 rounded px-1.5 py-0.5 bg-black/20">
							⌘K
						</span>
					</div>
				</div>

				{/* Notifications */}
				<button className="relative p-2 text-muted hover:text-white rounded-lg hover:bg-white/5 transition-colors">
					<Bell className="w-5 h-5" />
					<span className="absolute top-1.5 right-1.5 w-2 h-2 rounded-full bg-danger border-2 border-background"></span>
				</button>

				{/* User Profile */}
				<button className="ml-2 w-8 h-8 rounded-lg bg-gradient-to-br from-surface-2 to-surface border border-border flex items-center justify-center overflow-hidden hover:border-muted transition-colors">
					<div className="text-xs font-bold text-white bg-gradient-to-br from-accent to-accent-soft bg-clip-text text-transparent">
						JD
					</div>
				</button>
			</div>
		</header>
	);
}
