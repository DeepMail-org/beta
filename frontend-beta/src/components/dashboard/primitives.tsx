"use client";

import type { ReactNode } from "react";

/* ── Cards ─────────────────────────────────────────────── */
export function Card({
	title,
	subtitle,
	actions,
	className = "",
	children,
}: {
	title?: string;
	subtitle?: string;
	actions?: ReactNode;
	className?: string;
	children: ReactNode;
}) {
	return (
		<section className={`glass rounded-xl border border-border overflow-hidden flex flex-col ${className}`}>
			{(title || actions) && (
				<header className="px-5 py-4 border-b border-border flex items-center justify-between gap-4 bg-surface/50">
					<div className="min-w-0">
						{title && <h3 className="text-sm font-display font-semibold tracking-wide text-foreground">{title}</h3>}
						{subtitle && (
							<p className="text-xs text-muted mt-1 truncate">
								{subtitle}
							</p>
						)}
					</div>
					{actions && <div className="flex items-center gap-2 flex-shrink-0">{actions}</div>}
				</header>
			)}
			<div className="p-5 flex-1">{children}</div>
		</section>
	);
}

export function StatCard({
	label,
	value,
	change,
	trend = "up",
	icon,
}: {
	label: string;
	value: string;
	change?: string;
	trend?: "up" | "down" | "flat";
	icon?: ReactNode;
}) {
	const trendColor =
		trend === "up" ? "text-success" : trend === "down" ? "text-danger" : "text-muted";
	const arrow = trend === "up" ? "▲" : trend === "down" ? "▼" : "→";
	
	return (
		<div className="glass rounded-xl border border-border p-5 flex flex-col justify-between hover:border-border-hover transition-colors">
			<div className="flex items-center justify-between mb-3 text-muted">
				<span className="text-xs font-medium uppercase tracking-wider">{label}</span>
				{icon && <span className="text-muted">{icon}</span>}
			</div>
			<div className="text-3xl font-display font-semibold text-foreground tracking-tight">{value}</div>
			{change && (
				<div className={`text-xs mt-3 flex items-center gap-1.5 font-medium ${trendColor}`}>
					<span aria-hidden className="text-[10px]">{arrow}</span> {change}
				</div>
			)}
		</div>
	);
}

/* ── Inputs / Buttons ──────────────────────────────────── */
export function Btn({
	variant = "default",
	size = "md",
	children,
	className = "",
	...rest
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
	variant?: "default" | "accent" | "danger" | "success";
	size?: "sm" | "md";
}) {
	const baseClass = "inline-flex items-center justify-center font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-background disabled:opacity-50 disabled:pointer-events-none";
	
	const variantClasses = {
		default: "bg-surface-2 text-foreground border border-border hover:bg-surface-2/80 focus:ring-border",
		accent: "bg-accent text-white hover:bg-accent/90 focus:ring-accent",
		danger: "bg-danger text-white hover:bg-danger/90 focus:ring-danger",
		success: "bg-success text-white hover:bg-success/90 focus:ring-success",
	};
	
	const sizeClasses = {
		sm: "text-xs px-3 py-1.5",
		md: "text-sm px-4 py-2",
	};
	
	return (
		<button {...rest} className={`${baseClass} ${variantClasses[variant]} ${sizeClasses[size]} ${className}`.trim()}>
			{children}
		</button>
	);
}

export function Search({
	placeholder = "Search…",
	value,
	onChange,
}: {
	placeholder?: string;
	value?: string;
	onChange?: (v: string) => void;
}) {
	return (
		<div className="relative flex items-center w-full max-w-sm">
			<svg
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				strokeWidth={2}
				className="absolute left-3 w-4 h-4 text-muted pointer-events-none"
			>
				<circle cx="11" cy="11" r="7" />
				<path d="m20 20-3.5-3.5" strokeLinecap="round" />
			</svg>
			<input
				value={value ?? ""}
				onChange={(e) => onChange?.(e.target.value)}
				placeholder={placeholder}
				className="w-full bg-surface-2 border border-border text-foreground text-sm rounded-lg pl-9 pr-3 py-1.5 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent transition-colors"
			/>
		</div>
	);
}

/* ── Severity Badge ────────────────────────────────────── */
export type SeverityLevel = "critical" | "warning" | "caution" | "ok";
export function Severity({
	level,
	label,
}: {
	level: SeverityLevel;
	label?: string;
}) {
	const config = {
		critical: { bg: "bg-danger/10", text: "text-danger", border: "border-danger/20", defaultLabel: "Critical" },
		warning: { bg: "bg-warning/10", text: "text-warning", border: "border-warning/20", defaultLabel: "High" },
		caution: { bg: "bg-accent/10", text: "text-accent", border: "border-accent/20", defaultLabel: "Medium" },
		ok: { bg: "bg-success/10", text: "text-success", border: "border-success/20", defaultLabel: "Safe" },
	};
	
	const { bg, text, border, defaultLabel } = config[level];
	
	return (
		<span className={`inline-flex items-center px-2 py-0.5 rounded text-[11px] font-bold uppercase tracking-wider border ${bg} ${text} ${border}`}>
			{label ?? defaultLabel}
		</span>
	);
}

/* ── Tabs / Toggle / Tag ───────────────────────────────── */
export function Tabs({
	tabs,
	active,
	onChange,
}: {
	tabs: { id: string; label: string; count?: number }[];
	active: string;
	onChange: (id: string) => void;
}) {
	return (
		<div className="flex items-center gap-1 border-b border-border mb-4">
			{tabs.map((t) => {
				const isActive = active === t.id;
				return (
					<button
						key={t.id}
						type="button"
						onClick={() => onChange(t.id)}
						className={`relative px-4 py-2 text-sm font-medium transition-colors ${
							isActive ? "text-accent" : "text-muted hover:text-secondary"
						}`}
					>
						<span className="flex items-center gap-2">
							{t.label}
							{typeof t.count === "number" && (
								<span className={`text-[10px] px-1.5 py-0.5 rounded-full ${isActive ? 'bg-accent/10 text-accent' : 'bg-surface-2 text-muted'}`}>
									{t.count}
								</span>
							)}
						</span>
						{isActive && (
							<span className="absolute bottom-[-1px] left-0 w-full h-[2px] bg-accent rounded-t-full" />
						)}
					</button>
				);
			})}
		</div>
	);
}

export function Toggle({
	on,
	onChange,
	label,
}: {
	on: boolean;
	onChange: (v: boolean) => void;
	label?: string;
}) {
	return (
		<button
			type="button"
			role="switch"
			aria-checked={on}
			aria-label={label}
			onClick={() => onChange(!on)}
			className={`relative inline-flex h-5 w-9 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 focus:ring-offset-background ${
				on ? 'bg-accent' : 'bg-surface-2'
			}`}
		>
			<span
				aria-hidden="true"
				className={`pointer-events-none inline-block h-4 w-4 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${
					on ? 'translate-x-4' : 'translate-x-0'
				}`}
			/>
		</button>
	);
}

export function Tag({
	children,
	tone = "default",
}: {
	children: ReactNode;
	tone?: "default" | "danger" | "warn" | "ok";
}) {
	const config = {
		default: "bg-surface-2 text-secondary border border-border",
		danger: "bg-danger/10 text-danger border border-danger/20",
		warn: "bg-warning/10 text-warning border border-warning/20",
		ok: "bg-success/10 text-success border border-success/20",
	};
	
	return (
		<span className={`inline-flex items-center px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-wider ${config[tone]}`}>
			{children}
		</span>
	);
}

/* ── Charts ───────────────────────────────────────────── */
export function LineChart({
	data,
	height = 180,
}: {
	data: { label: string; value: number }[];
	height?: number;
}) {
	if (!data.length) return null;
	const w = 600;
	const h = height;
	const pad = { l: 28, r: 12, t: 12, b: 22 };
	const innerW = w - pad.l - pad.r;
	const innerH = h - pad.t - pad.b;
	const max = Math.max(...data.map((d) => d.value), 1);
	const step = innerW / Math.max(data.length - 1, 1);
	const pts = data.map((d, i) => {
		const x = pad.l + i * step;
		const y = pad.t + innerH - (d.value / max) * innerH;
		return [x, y] as const;
	});
	const path = pts
		.map(([x, y], i) => (i === 0 ? `M${x},${y}` : `L${x},${y}`))
		.join(" ");
	const area = `${path} L${pts[pts.length - 1][0]},${pad.t + innerH} L${pts[0][0]},${pad.t + innerH} Z`;
	return (
		<svg
			viewBox={`0 0 ${w} ${h}`}
			className="w-full"
			preserveAspectRatio="none"
			role="img"
			aria-label="Line chart"
		>
			<defs>
				<linearGradient id="lineFill" x1="0" x2="0" y1="0" y2="1">
					<stop offset="0%" stopColor="#fff" stopOpacity="0.18" />
					<stop offset="100%" stopColor="#fff" stopOpacity="0" />
				</linearGradient>
			</defs>
			{[0, 0.25, 0.5, 0.75, 1].map((t) => {
				const y = pad.t + innerH * t;
				return (
					<line
						key={t}
						x1={pad.l}
						x2={w - pad.r}
						y1={y}
						y2={y}
						stroke="rgba(255,255,255,0.05)"
						strokeWidth={1}
					/>
				);
			})}
			<path d={area} fill="url(#lineFill)" />
			<path
				d={path}
				fill="none"
				stroke="rgba(255,255,255,0.85)"
				strokeWidth={1.6}
				strokeLinejoin="round"
			/>
			{pts.map(([x, y], i) => (
				<circle
					key={i}
					cx={x}
					cy={y}
					r={2}
					fill="#fff"
					opacity={i === pts.length - 1 ? 1 : 0.5}
				/>
			))}
			{data.map((d, i) => (
				<text
					key={i}
					x={pad.l + i * step}
					y={h - 6}
					fontSize={9}
					fill="rgba(255,255,255,0.4)"
					textAnchor="middle"
				>
					{d.label}
				</text>
			))}
		</svg>
	);
}

export function BarChart({
	data,
	height = 160,
}: {
	data: { label: string; value: number; tone?: "default" | "danger" | "warn" }[];
	height?: number;
}) {
	const w = 600;
	const h = height;
	const pad = { l: 24, r: 12, t: 10, b: 22 };
	const innerW = w - pad.l - pad.r;
	const innerH = h - pad.t - pad.b;
	const max = Math.max(...data.map((d) => d.value), 1);
	const bw = (innerW / data.length) * 0.6;
	const gap = (innerW / data.length) * 0.4;
	return (
		<svg
			viewBox={`0 0 ${w} ${h}`}
			className="w-full"
			preserveAspectRatio="none"
			role="img"
			aria-label="Bar chart"
		>
			{data.map((d, i) => {
				const bh = (d.value / max) * innerH;
				const x = pad.l + i * (bw + gap) + gap / 2;
				const y = pad.t + innerH - bh;
				const color =
					d.tone === "danger"
						? "#f87171"
						: d.tone === "warn"
							? "#fbbf24"
							: "rgba(255,255,255,0.78)";
				return (
					<g key={i}>
						<rect
							x={x}
							y={y}
							width={bw}
							height={bh}
							rx={2}
							fill={color}
							opacity={0.85}
						/>
						<text
							x={x + bw / 2}
							y={h - 6}
							fontSize={9}
							fill="rgba(255,255,255,0.4)"
							textAnchor="middle"
						>
							{d.label}
						</text>
					</g>
				);
			})}
		</svg>
	);
}

export function Donut({
	segments,
	size = 160,
	thickness = 22,
	centerLabel,
	centerValue,
}: {
	segments: { label: string; value: number; color: string }[];
	size?: number;
	thickness?: number;
	centerLabel?: string;
	centerValue?: string;
}) {
	const total = segments.reduce((s, x) => s + x.value, 0) || 1;
	const r = (size - thickness) / 2;
	const c = size / 2;
	const circ = 2 * Math.PI * r;
	let offset = 0;
	return (
		<div className="flex items-center gap-6">
			<svg
				width={size}
				height={size}
				viewBox={`0 0 ${size} ${size}`}
				role="img"
				aria-label="Donut chart"
			>
				<circle
					cx={c}
					cy={c}
					r={r}
					fill="none"
					stroke="rgba(255,255,255,0.06)"
					strokeWidth={thickness}
				/>
				{segments.map((s, i) => {
					const len = (s.value / total) * circ;
					const dash = `${len} ${circ - len}`;
					const el = (
						<circle
							key={i}
							cx={c}
							cy={c}
							r={r}
							fill="none"
							stroke={s.color}
							strokeWidth={thickness}
							strokeDasharray={dash}
							strokeDashoffset={-offset}
							transform={`rotate(-90 ${c} ${c})`}
							strokeLinecap="butt"
						/>
					);
					offset += len;
					return el;
				})}
				{(centerValue || centerLabel) && (
					<g textAnchor="middle">
						<text
							x={c}
							y={c - 2}
							fontSize={20}
							fontWeight={700}
							fill="#fff"
						>
							{centerValue}
						</text>
						<text
							x={c}
							y={c + 16}
							fontSize={9}
							fill="rgba(255,255,255,0.5)"
							style={{ letterSpacing: 1 }}
						>
							{centerLabel}
						</text>
					</g>
				)}
			</svg>
			<ul className="space-y-2 text-xs">
				{segments.map((s, i) => (
					<li key={i} className="flex items-center gap-2">
						<span
							className="inline-block w-2.5 h-2.5 rounded-sm"
							style={{ background: s.color }}
						/>
						<span className="opacity-70">{s.label}</span>
						<span className="ml-auto font-bold">
							{Math.round((s.value / total) * 100)}%
						</span>
					</li>
				))}
			</ul>
		</div>
	);
}

export function RadarChart({
	axes,
	size = 220,
}: {
	axes: { label: string; value: number }[];
	size?: number;
}) {
	const c = size / 2;
	const r = c - 24;
	const n = axes.length;
	const angle = (i: number) => -Math.PI / 2 + (2 * Math.PI * i) / n;
	const point = (i: number, v: number) => {
		const rad = r * Math.max(0, Math.min(1, v));
		return [c + rad * Math.cos(angle(i)), c + rad * Math.sin(angle(i))] as const;
	};
	const polyPts = axes.map((a, i) => point(i, a.value).join(",")).join(" ");
	return (
		<svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} role="img" aria-label="Radar chart">
			{[0.25, 0.5, 0.75, 1].map((t, i) => (
				<polygon
					key={i}
					points={axes
						.map((_, idx) => point(idx, t).join(","))
						.join(" ")}
					fill="none"
					stroke="rgba(255,255,255,0.06)"
					strokeWidth={1}
				/>
			))}
			{axes.map((_, i) => {
				const [x, y] = point(i, 1);
				return (
					<line
						key={i}
						x1={c}
						y1={c}
						x2={x}
						y2={y}
						stroke="rgba(255,255,255,0.06)"
					/>
				);
			})}
			<polygon
				points={polyPts}
				fill="rgba(255,255,255,0.10)"
				stroke="rgba(255,255,255,0.85)"
				strokeWidth={1.5}
			/>
			{axes.map((a, i) => {
				const [x, y] = point(i, 1.18);
				return (
					<text
						key={i}
						x={x}
						y={y}
						fontSize={9}
						fill="rgba(255,255,255,0.55)"
						textAnchor="middle"
						dominantBaseline="middle"
					>
						{a.label}
					</text>
				);
			})}
		</svg>
	);
}

export function GeoMap({
	hotspots = [],
}: {
	hotspots?: { x: number; y: number; intensity: number; label?: string }[];
}) {
	return (
		<div className="relative w-full" style={{ aspectRatio: "2 / 1" }}>
			<svg
				viewBox="0 0 800 400"
				className="w-full h-full"
				preserveAspectRatio="xMidYMid meet"
				role="img"
				aria-label="Geo threat map"
			>
				<defs>
					<pattern
						id="grid"
						width="40"
						height="40"
						patternUnits="userSpaceOnUse"
					>
						<path
							d="M 40 0 L 0 0 0 40"
							fill="none"
							stroke="rgba(255,255,255,0.04)"
							strokeWidth="0.5"
						/>
					</pattern>
				</defs>
				<rect width="800" height="400" fill="url(#grid)" />
				<path
					d="M50 180 Q 120 140 180 160 T 320 180 Q 380 200 460 180 T 620 200 Q 700 220 760 200"
					fill="none"
					stroke="rgba(255,255,255,0.10)"
					strokeWidth="1.5"
				/>
				<path
					d="M80 240 Q 160 220 240 240 T 420 250 Q 520 240 620 260 T 750 270"
					fill="none"
					stroke="rgba(255,255,255,0.06)"
					strokeWidth="1.5"
				/>
				{hotspots.map((h, i) => {
					const radius = 4 + h.intensity * 8;
					return (
						<g key={i}>
							<circle
								cx={h.x}
								cy={h.y}
								r={radius * 2}
								fill="rgba(248,113,113,0.15)"
							>
								<animate
									attributeName="r"
									from={radius}
									to={radius * 2.5}
									dur="2.4s"
									repeatCount="indefinite"
								/>
								<animate
									attributeName="opacity"
									from="0.4"
									to="0"
									dur="2.4s"
									repeatCount="indefinite"
								/>
							</circle>
							<circle
								cx={h.x}
								cy={h.y}
								r={radius}
								fill="#f87171"
								opacity="0.8"
							/>
						</g>
					);
				})}
			</svg>
		</div>
	);
}
