"use client";

import { useState, useEffect } from "react";

function getClockTime(): string {
	const now = new Date();
	const ms = String(now.getMilliseconds()).padStart(3, "0");
	return `${now.toLocaleTimeString("en-US", { hour12: false })}.${ms}`;
}

export default function SandboxPage() {
	const [logs, setLogs] = useState<string[]>([]);
	const [clockTime, setClockTime] = useState("00:00:00.000");

	useEffect(() => {
		setClockTime(getClockTime());
		const clockInterval = setInterval(() => {
			setClockTime(getClockTime());
		}, 200);

		return () => clearInterval(clockInterval);
	}, []);

	useEffect(() => {
		setLogs([]);
		const bootLogs = [
			"[SYS] Initializing Sentinel Sandbox Environment v2.4.1",
			"[SYS] Allocating isolated memory pool...",
			"[OK] Memory pool allocated: 4096MB",
			"[SYS] Loading Windows 11 API stubs...",
			"[OK] API Hooks injected successfully.",
			"[NET] Disabling external network routing...",
			"[OK] Network isolated (loopback only).",
			"[SCAN] Standing by for detonation payload (Awaiting queue...)",
		];

		let k = 0;
		const interval = setInterval(() => {
			if (k < bootLogs.length) {
				const nextLog = bootLogs[k];
				setLogs((prev) =>
					prev.includes(nextLog) ? prev : [...prev, nextLog],
				);
				k++;
			} else {
				clearInterval(interval);
			}
		}, 800);

		return () => clearInterval(interval);
	}, []);

	return (
		<div className="p-8 lg:p-12 space-y-8 max-w-6xl mx-auto">
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-4xl font-bold text-on-surface tracking-tight font-headline">
						DYNAMIC_DETONATION{" "}
						<span className="text-secondary">Sandbox</span>
					</h2>
					<p className="text-on-surface-variant mt-2 text-sm">
						Real-time isolated environment for analyzing execution
						behavior.
					</p>
				</div>
				<div className="flex items-center gap-4">
					<div className="flex items-center gap-2 px-4 py-2 bg-surface-container-low border border-outline-variant/20 rounded-lg">
						<span className="w-2 h-2 rounded-full bg-tertiary animate-pulse" />
						<span className="text-[10px] uppercase tracking-widest font-bold text-on-surface">
							Cluster Online
						</span>
					</div>
					<button className="px-4 py-2 bg-secondary/10 text-secondary border border-secondary/20 hover:bg-secondary/20 font-bold text-xs uppercase tracking-widest rounded transition-colors">
						Force Reset
					</button>
				</div>
			</div>

			<div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
				<div className="lg:col-span-2 glass-panel rounded-xl overflow-hidden flex flex-col h-[500px]">
					<div className="bg-bg border-b border-outline-variant/10 px-4 py-3 flex items-center justify-between">
						<div className="flex items-center gap-2">
							<span className="material-symbols-outlined text-outline text-sm">
								terminal
							</span>
							<span className="text-xs font-mono text-outline font-bold">
								sentinel_tty1
							</span>
						</div>
					</div>
					<div className="p-6 bg-black/40 flex-1 overflow-y-auto font-mono text-xs space-y-2">
						{logs.map((log, idx) => (
							<div key={idx} className="flex gap-4">
								<span className="text-outline-variant select-none opacity-50">
									{clockTime}
								</span>
								<span
									className={`${log.startsWith("[OK]") ? "text-tertiary" : log.startsWith("[SCAN]") ? "text-primary" : "text-on-surface-variant"}`}
								>
									{log}
								</span>
							</div>
						))}
						<div className="flex gap-4 animate-pulse">
							<span className="text-outline-variant select-none opacity-50">
								{clockTime}
							</span>
							<span className="text-primary-container">_</span>
						</div>
					</div>
				</div>

				<div className="space-y-6">
					<div className="glass-panel p-6 rounded-xl">
						<h3 className="text-sm font-bold tracking-widest uppercase font-headline mb-4">
							VM Metrics
						</h3>
						<div className="space-y-4">
							<div>
								<div className="flex justify-between text-xs mb-1">
									<span className="text-outline">
										CPU Usage
									</span>
									<span className="text-on-surface font-bold">
										12%
									</span>
								</div>
								<div className="h-1 bg-surface-container-highest rounded-full overflow-hidden">
									<div className="w-[12%] h-full bg-primary" />
								</div>
							</div>
							<div>
								<div className="flex justify-between text-xs mb-1">
									<span className="text-outline">
										RAM Usage
									</span>
									<span className="text-on-surface font-bold">
										1.2GB / 4.0GB
									</span>
								</div>
								<div className="h-1 bg-surface-container-highest rounded-full overflow-hidden">
									<div className="w-[30%] h-full bg-primary-container" />
								</div>
							</div>
						</div>
					</div>

					<div className="glass-panel p-6 rounded-xl">
						<h3 className="text-sm font-bold tracking-widest uppercase font-headline mb-4">
							Active Modules
						</h3>
						<div className="space-y-2 text-xs">
							<div className="flex items-center justify-between p-2 rounded bg-surface-container-low/50 border border-outline-variant/10">
								<span className="text-on-surface">
									Filesystem Hooks
								</span>
								<span className="material-symbols-outlined text-tertiary text-sm">
									check_circle
								</span>
							</div>
							<div className="flex items-center justify-between p-2 rounded bg-surface-container-low/50 border border-outline-variant/10">
								<span className="text-on-surface">
									Registry Monitor
								</span>
								<span className="material-symbols-outlined text-tertiary text-sm">
									check_circle
								</span>
							</div>
							<div className="flex items-center justify-between p-2 rounded bg-surface-container-low/50 border border-outline-variant/10">
								<span className="text-on-surface">
									Network Pcap
								</span>
								<span className="material-symbols-outlined text-tertiary text-sm">
									check_circle
								</span>
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
