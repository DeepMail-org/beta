"use client";

import { useEffect, useState } from "react";
import { getStoredToken } from "@/lib/api";

export default function TopBar() {
	const [hasToken, setHasToken] = useState(false);

	useEffect(() => {
		const checkToken = () => setHasToken(Boolean(getStoredToken()));
		checkToken();

		// Listen for storage changes (other tabs)
		window.addEventListener("storage", checkToken);
		// Listen for manual updates (same tab)
		window.addEventListener("storage_local_update", checkToken);

		return () => {
			window.removeEventListener("storage", checkToken);
			window.removeEventListener("storage_local_update", checkToken);
		};
	}, []);

	return (
		<header className="sticky top-0 z-40 bg-bg/80 backdrop-blur-xl shadow-[0_4px_20px_rgba(196,154,255,0.08)] flex justify-between items-center w-full px-8 py-4">
			<div className="flex items-center gap-8">
				{/* Brand */}
				<span className="text-2xl font-bold tracking-tighter text-primary font-headline uppercase">
					DeepMail
				</span>
			</div>
			<div className="flex items-center gap-4">
				<div className="bg-surface-container-lowest px-4 py-1.5 rounded-full flex items-center gap-2 border border-outline-variant/10">
					<span
						className={`w-2 h-2 rounded-full ${hasToken ? "bg-tertiary" : "bg-dracula-red"}`}
					/>
					<span
						className={`text-[10px] font-bold uppercase tracking-widest ${hasToken ? "text-tertiary" : "text-dracula-red"}`}
					>
						{hasToken ? "Connected" : "Disconnected"}
					</span>
				</div>
				{/* System Status */}
				<div className="bg-surface-container-lowest px-4 py-1.5 rounded-full flex items-center gap-2 border border-outline-variant/10">
					<span className="w-2 h-2 rounded-full bg-tertiary animate-pulse" />
					<span className="text-[10px] font-bold uppercase tracking-widest text-tertiary">
						System Live
					</span>
				</div>
				{/* Profile */}
				<button className="text-on-surface-variant hover:text-secondary transition-colors duration-300 active:scale-95">
					<span className="material-symbols-outlined">
						account_circle
					</span>
				</button>
			</div>
		</header>
	);
}
