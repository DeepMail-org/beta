"use client";

import { useMemo, useState } from "react";
import { clearToken, getStoredToken, setToken } from "@/lib/api";

export default function SettingsPage() {
	const [tokenInput, setTokenInput] = useState(() => getStoredToken() || "");
	const [saved, setSaved] = useState(false);
	const [cleared, setCleared] = useState(false);

	const hasToken = useMemo(() => tokenInput.trim().length > 0, [tokenInput]);

	const onSave = () => {
		const trimmed = tokenInput.trim();
		if (!trimmed) return;
		setToken(trimmed);
		setSaved(true);
		setCleared(false);
		setTimeout(() => setSaved(false), 2000);
	};

	const onClear = () => {
		clearToken();
		setTokenInput("");
		setCleared(true);
		setSaved(false);
		setTimeout(() => setCleared(false), 2000);
	};

	return (
		<div className="flex flex-col gap-6 w-full max-w-5xl mx-auto">
			<div>
				<h2 className="text-4xl font-bold text-on-surface tracking-tight font-headline">
					SYSTEM_CONFIG <span className="text-primary">Settings</span>
				</h2>
				<p className="text-on-surface-variant mt-2 text-sm">
					Configure analysis engine parameters and integration settings.
				</p>
			</div>

			<div className="grid grid-cols-1 md:grid-cols-3 gap-8">
				<div className="md:col-span-1 space-y-2">
					<button className="w-full text-left px-4 py-3 bg-primary/10 text-primary border border-primary/20 rounded-lg font-bold text-sm tracking-wide">
						General Preferences
					</button>
					<button className="w-full text-left px-4 py-3 hover:bg-surface-container/50 text-on-surface-variant rounded-lg font-bold text-sm tracking-wide transition-colors">
						API Keys
					</button>
					<button className="w-full text-left px-4 py-3 hover:bg-surface-container/50 text-on-surface-variant rounded-lg font-bold text-sm tracking-wide transition-colors">
						Sandbox Connectors
					</button>
					<button className="w-full text-left px-4 py-3 hover:bg-surface-container/50 text-on-surface-variant rounded-lg font-bold text-sm tracking-wide transition-colors">
						Notifications
					</button>
				</div>

				<div className="md:col-span-2 space-y-8">
					<div className="glass-panel p-8 rounded-xl space-y-6">
						<div>
							<h3 className="text-lg font-bold text-on-surface font-headline">
								Local API Authentication (Manual Token)
							</h3>
							<p className="text-xs text-outline mt-1">
								Paste your JWT for dashboard/upload/results requests.
							</p>
						</div>

						<div className="space-y-3">
							<label className="text-xs font-bold uppercase tracking-widest text-on-surface">
								Bearer Token
							</label>
							<textarea
								value={tokenInput}
								onChange={(e) => setTokenInput(e.target.value)}
								placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
								className="w-full min-h-28 rounded-lg bg-surface-container-low border border-outline-variant/30 px-3 py-2 text-xs text-on-surface font-mono focus:outline-none focus:border-primary/50"
							/>
							<div className="flex items-center gap-2 text-[11px]">
								<span
									className={`inline-block w-2 h-2 rounded-full ${hasToken ? "bg-tertiary" : "bg-dracula-red"}`}
								/>
								<span className="text-on-surface-variant">
									{hasToken
										? "Token present"
										: "No token configured"}
								</span>
							</div>
						</div>

						<div className="flex flex-wrap items-center gap-3">
							<button
								onClick={onSave}
								disabled={!hasToken}
								className="px-6 py-2 bg-primary text-on-primary font-bold text-xs uppercase tracking-widest rounded hover:bg-primary/90 transition-colors disabled:opacity-50"
							>
								Save Token
							</button>
							<button
								onClick={onClear}
								className="px-6 py-2 bg-dracula-red/10 text-dracula-red border border-dracula-red/30 font-bold text-xs uppercase tracking-widest rounded hover:bg-dracula-red/20 transition-colors"
							>
								Clear Token
							</button>
							{saved && (
								<span className="text-xs text-tertiary font-bold">
									Token saved.
								</span>
							)}
							{cleared && (
								<span className="text-xs text-dracula-red font-bold">
									Token cleared.
								</span>
							)}
						</div>
					</div>
				</div>
			</div>
		</div>
	);
}
