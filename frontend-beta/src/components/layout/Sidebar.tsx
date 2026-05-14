"use client";

import React, { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import {
	LayoutDashboard,
	ShieldAlert,
	Briefcase,
	Radio,
	TerminalSquare,
	Settings,
	HelpCircle,
	ChevronRight,
	Bot,
} from "lucide-react";
import { motion } from "framer-motion";

const navItems = [
	{ name: "Dashboard", href: "/dashboard", icon: LayoutDashboard },
	{ name: "Threat Analysis", href: "/analysis", icon: ShieldAlert },
	{ name: "Cases", href: "/cases", icon: Briefcase },
	{ name: "Threat Intel", href: "/threat-intel", icon: Radio },
	{ name: "Sandbox", href: "/sandbox", icon: TerminalSquare },
];

const bottomNavItems = [
	{ name: "Settings", href: "/settings", icon: Settings },
	{ name: "Support", href: "/support", icon: HelpCircle },
];

export function Sidebar() {
	const pathname = usePathname();
	const [isOpen, setIsOpen] = useState(false);

	return (
		<>
			{/* Mobile Overlay */}
			{isOpen && (
				<div
					className="fixed inset-0 bg-black/50 backdrop-blur-sm z-40 md:hidden"
					onClick={() => setIsOpen(false)}
				/>
			)}

			<aside
				className={`
					fixed md:sticky top-0 left-0 h-screen z-50
					w-64 flex-shrink-0 flex flex-col
					bg-[#0e0f11] border-r border-[rgba(255,200,60,0.08)]
					transition-transform duration-300 ease-in-out
					${isOpen ? "translate-x-0" : "-translate-x-full md:translate-x-0"}
				`}
			>
				{/* Logo Area */}
				<div className="p-4 flex items-center justify-between">
					<Link href="/dashboard" className="flex items-center gap-3 px-2 py-1.5 rounded-lg hover:bg-white/5 transition-colors group">
						<div className="w-8 h-8 rounded-lg bg-gradient-to-br from-accent to-accent-soft flex items-center justify-center shadow-lg shadow-accent/20">
							<ShieldAlert className="w-4 h-4 text-white" />
						</div>
						<span className="font-display font-bold text-lg tracking-tight text-white group-hover:text-accent-soft transition-colors">
							DeepMail
						</span>
					</Link>
					
					{/* Mobile Close Button */}
					<button 
						className="md:hidden p-2 text-muted hover:text-white rounded-lg hover:bg-white/5"
						onClick={() => setIsOpen(false)}
					>
						<ChevronRight className="w-5 h-5 rotate-180" />
					</button>
				</div>

				{/* Main Navigation */}
				<motion.nav 
					className="flex-1 px-3 py-4 space-y-1 overflow-y-auto"
					initial="hidden"
					animate="visible"
					variants={{
						hidden: {},
						visible: { transition: { staggerChildren: 0.05 } }
					}}
				>
					<div className="text-xs font-semibold text-muted/50 uppercase tracking-wider px-3 mb-2">
						Platform
					</div>
					{navItems.map((item) => {
						const isActive = pathname.startsWith(item.href);
						return (
							<motion.div key={item.href} variants={{
								hidden: { opacity: 0, x: -10 },
								visible: { opacity: 1, x: 0 }
							}}>
								<Link
									href={item.href}
									className={`
										flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium
										transition-all duration-200 relative group
										${isActive 
											? "text-white bg-white/10" 
											: "text-muted hover:text-secondary hover:bg-white/5"
										}
									`}
								>
									{isActive && (
										<motion.div 
											layoutId="sidebar-active" 
											className="absolute left-0 top-1.5 bottom-1.5 w-1 bg-accent rounded-r-full" 
											initial={{ opacity: 0 }}
											animate={{ opacity: 1 }}
											transition={{ duration: 0.2 }}
										/>
									)}
									<item.icon className={`w-4 h-4 ${isActive ? "text-accent" : "text-muted group-hover:text-secondary"}`} />
									{item.name}
								</Link>
							</motion.div>
						);
					})}
				</motion.nav>

				{/* Bottom Section */}
				<motion.div 
					className="p-3 border-t border-[rgba(255,200,60,0.08)] space-y-1"
					initial="hidden"
					animate="visible"
					variants={{
						hidden: {},
						visible: { transition: { staggerChildren: 0.05, delayChildren: 0.2 } }
					}}
				>
					<motion.button 
						variants={{
							hidden: { opacity: 0, y: 10 },
							visible: { opacity: 1, y: 0 }
						}}
						className="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-lg text-sm font-medium text-white bg-gradient-to-br from-white/10 to-white/5 border border-white/10 hover:border-white/20 hover:from-white/15 hover:to-white/5 transition-all shadow-sm"
					>
						<Bot className="w-4 h-4 text-accent-soft" />
						Ask AI Assistant
					</motion.button>
					
					<div className="h-2" />
					
					{bottomNavItems.map((item) => {
						const isActive = pathname.startsWith(item.href);
						return (
							<motion.div key={item.href} variants={{
								hidden: { opacity: 0, x: -10 },
								visible: { opacity: 1, x: 0 }
							}}>
								<Link
									href={item.href}
									className={`
										flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium
										transition-all duration-200
										${isActive 
											? "text-white bg-white/10" 
											: "text-muted hover:text-secondary hover:bg-white/5"
										}
									`}
								>
									<item.icon className="w-4 h-4" />
									{item.name}
								</Link>
							</motion.div>
						);
					})}
				</motion.div>
			</aside>
		</>
	);
}
