"use client";

import { useState, useEffect } from "react";
import { Card } from "@/components/dashboard/primitives";
import { Terminal, CheckCircle2 } from "lucide-react";

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
		<div className="flex flex-col gap-6 w-full max-w-7xl mx-auto">
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-3xl font-display font-bold tracking-tight text-foreground">
						DYNAMIC_DETONATION{" "}
						<span className="text-secondary">Sandbox</span>
					</h2>
					<p className="text-muted mt-1 text-sm">
						Real-time isolated environment for analyzing execution behavior.
					</p>
				</div>
				<div className="flex items-center gap-4">
					<div className="flex items-center gap-2 px-3 py-1.5 bg-surface-2 rounded-lg border border-border">
						<span className="w-2 h-2 rounded-full bg-success animate-pulse" />
						<span className="text-[10px] uppercase tracking-widest font-bold text-foreground">
							Cluster Online
						</span>
					</div>
					<button className="px-4 py-2 bg-secondary/10 text-secondary border border-secondary/20 hover:bg-secondary/20 font-bold text-xs uppercase tracking-widest rounded transition-colors">
						Force Reset
					</button>
				</div>
			</div>

			<div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
				<div className="lg:col-span-2 glass rounded-xl overflow-hidden flex flex-col h-[500px] border border-border">
					<div className="bg-surface border-b border-border px-4 py-3 flex items-center justify-between">
						<div className="flex items-center gap-2">
							<Terminal className="w-4 h-4 text-muted" />
							<span className="text-xs font-mono text-muted font-bold">
								sentinel_tty1
							</span>
						</div>
					</div>
					<div className="p-6 bg-black/60 flex-1 overflow-y-auto font-mono text-xs space-y-2">
						{logs.map((log, idx) => (
							<div key={idx} className="flex gap-4">
								<span className="text-muted select-none opacity-50">
									{clockTime}
								</span>
								<span
									className={`${log.startsWith("[OK]") ? "text-success" : log.startsWith("[SCAN]") ? "text-accent" : "text-muted"}`}
								>
									{log}
								</span>
							</div>
						))}
						<div className="flex gap-4 animate-pulse">
							<span className="text-muted select-none opacity-50">
								{clockTime}
							</span>
							<span className="text-accent">_</span>
						</div>
					</div>
				</div>

				<div className="space-y-6">
					<Card title="VM Metrics">
						<div className="space-y-5">
							<div>
								<div className="flex justify-between text-xs mb-1.5">
									<span className="text-muted">CPU Usage</span>
									<span className="text-foreground font-bold">12%</span>
								</div>
								<div className="h-1.5 bg-surface-2 rounded-full overflow-hidden">
									<div className="w-[12%] h-full bg-accent" />
								</div>
							</div>
							<div>
								<div className="flex justify-between text-xs mb-1.5">
									<span className="text-muted">RAM Usage</span>
									<span className="text-foreground font-bold">1.2GB / 4.0GB</span>
								</div>
								<div className="h-1.5 bg-surface-2 rounded-full overflow-hidden">
									<div className="w-[30%] h-full bg-accent-soft" />
								</div>
							</div>
						</div>
					</Card>

					<Card title="Active Modules">
						<div className="space-y-2 text-xs">
							<div className="flex items-center justify-between p-2.5 rounded-lg bg-surface-2 border border-border">
								<span className="text-foreground font-medium">Filesystem Hooks</span>
								<CheckCircle2 className="w-4 h-4 text-success" />
							</div>
							<div className="flex items-center justify-between p-2.5 rounded-lg bg-surface-2 border border-border">
								<span className="text-foreground font-medium">Registry Monitor</span>
								<CheckCircle2 className="w-4 h-4 text-success" />
							</div>
							<div className="flex items-center justify-between p-2.5 rounded-lg bg-surface-2 border border-border">
								<span className="text-foreground font-medium">Network Pcap</span>
								<CheckCircle2 className="w-4 h-4 text-success" />
							</div>
						</div>
					</Card>
				</div>
			</div>
		</div>
	);
}
