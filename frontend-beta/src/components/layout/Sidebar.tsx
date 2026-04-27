"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const NAV_ITEMS = [
	{ href: "/", icon: "dashboard", label: "Dashboard" },
	{ href: "/upload", icon: "cloud_upload", label: "Upload" },
	{ href: "/analysis", icon: "analytics", label: "Analysis" },
	{ href: "/sandbox", icon: "biotech", label: "Sandbox" },
	{ href: "/reports", icon: "description", label: "Reports" },
	{ href: "/settings", icon: "settings", label: "Settings" },
];

export default function Sidebar() {
	const pathname = usePathname();

	const isActive = (href: string) => {
		if (href === "/") return pathname === "/";
		return pathname.startsWith(href);
	};

	return (
		<aside className="h-screen w-64 fixed left-0 top-0 bg-surface-container-low border-r border-primary/[0.12] shadow-2xl shadow-primary/5 flex flex-col py-8 z-50">
			{/* Logo */}
			<div className="px-6 mb-10">
				<h1 className="text-xl font-black text-primary font-[family-name:var(--font-headline)] tracking-tight">
					DeepMail SOC
				</h1>
				<p className="text-[10px] uppercase tracking-[0.2em] text-on-surface-variant mt-1">
					Synthetic Sentinel Active
				</p>
			</div>

			{/* Navigation */}
			<nav className="flex-1 space-y-1">
				{NAV_ITEMS.map((item) => (
					<Link
						key={item.href}
						href={item.href}
						className={`flex items-center px-6 py-3 transition-all duration-300 group ${
							isActive(item.href)
								? "text-primary bg-primary/10 border-r-2 border-primary"
								: "text-on-surface-variant hover:text-on-surface hover:bg-surface-container transition-all"
						}`}
					>
						<span
							className={`material-symbols-outlined mr-3 ${!isActive(item.href) ? "group-hover:translate-x-1 transition-transform" : ""}`}
						>
							{item.icon}
						</span>
						<span className="font-[family-name:var(--font-headline)] uppercase tracking-[0.05em] text-xs font-bold">
							{item.label}
						</span>
					</Link>
				))}
			</nav>

			{/* New Scan CTA */}
			<div className="px-6 mb-6">
				<Link
					href="/upload"
					className="block w-full text-center py-3 gradient-primary text-on-primary rounded-lg font-black text-xs uppercase tracking-widest shadow-[0_4px_20px_rgba(189,147,249,0.3)] hover:scale-[1.02] transition-transform"
				>
					NEW SCAN
				</Link>
			</div>

			{/* User Profile */}
			<div className="px-6">
				<div className="flex items-center p-3 bg-surface-container rounded-xl border border-outline-variant/10">
					<div className="w-10 h-10 rounded-full gradient-primary flex items-center justify-center text-on-primary font-bold text-sm">
						OP
					</div>
					<div className="ml-3">
						<p className="text-xs font-bold text-on-surface">
							SOC_OPERATOR
						</p>
						<p className="text-[10px] text-tertiary">
							LVL 4 CLEARANCE
						</p>
					</div>
				</div>
			</div>
		</aside>
	);
}
