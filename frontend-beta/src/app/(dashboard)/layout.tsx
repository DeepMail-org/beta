"use client";

import React from "react";
import { usePathname } from "next/navigation";
import { motion, AnimatePresence } from "framer-motion";
import { Sidebar } from "../../components/layout/Sidebar";
import { TopNav } from "../../components/layout/TopNav";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
	const pathname = usePathname();

	return (
		<div className="flex min-h-screen bg-background text-foreground font-sans">
			<Sidebar />
			
			<div className="flex flex-col flex-1 min-w-0">
				<TopNav />
				<main className="flex-1 p-4 lg:p-8 overflow-y-auto overflow-x-hidden">
					<AnimatePresence mode="wait">
						<motion.div
							key={pathname}
							initial={{ opacity: 0, y: 10 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -10 }}
							transition={{ duration: 0.2, ease: "easeOut" }}
							className="mx-auto max-w-7xl h-full"
						>
							{children}
						</motion.div>
					</AnimatePresence>
				</main>
			</div>
		</div>
	);
}
