"use client";

import {
	Btn,
	Card,
	GeoMap,
	Severity,
	StatCard,
	Tag,
} from "@/components/dashboard/primitives";
import { LineChartComponent } from "@/components/ui/line-chart";
import { RadarChart } from "@/components/ui/radar-chart";
import { DonutChart } from "@/components/ui/donut-chart";
import { BarChart } from "@/components/ui/bar-chart";
import { InteractiveMap } from "@/components/ui/interactive-map";

const TIMELINE = [
	{ label: "Mon", value: 142 },
	{ label: "Tue", value: 168 },
	{ label: "Wed", value: 199 },
	{ label: "Thu", value: 174 },
	{ label: "Fri", value: 231 },
	{ label: "Sat", value: 184 },
	{ label: "Sun", value: 257 },
];

const SEVERITY_SEGMENTS = [
	{ label: "Critical", value: 14, color: "var(--color-danger)" },
	{ label: "High", value: 27, color: "var(--color-warning)" },
	{ label: "Medium", value: 41, color: "var(--color-accent-soft)" },
	{ label: "Safe", value: 218, color: "var(--color-success)" },
];

const VECTOR_AXES = [
	{ label: "Phishing", value: 0.92 },
	{ label: "Malware", value: 0.64 },
	{ label: "BEC", value: 0.78 },
	{ label: "Spoofing", value: 0.55 },
	{ label: "Spam", value: 0.42 },
	{ label: "Recon", value: 0.31 },
];

const TOP_SENDERS = [
	{ rank: 1, label: "no-reply@apple-id.support", value: 47 },
	{ rank: 2, label: "billing@paypal-secure.io", value: 38 },
	{ rank: 3, label: "ceo@acme-corp.tk", value: 29 },
	{ rank: 4, label: "alerts@dropbox-share.app", value: 24 },
	{ rank: 5, label: "noreply@office365-login.io", value: 19 },
];

const HOTSPOTS = [
	{ lat: 55.75, lng: 37.61, intensity: 1.0, label: "RU" },
	{ lat: 35.86, lng: 104.19, intensity: 0.7, label: "CN" },
	{ lat: 32.42, lng: 53.68, intensity: 0.5, label: "IR" },
	{ lat: -0.78, lng: 113.92, intensity: 0.4, label: "ID" },
	{ lat: -14.23, lng: -51.92, intensity: 0.3, label: "BR" },
];

const RECENT_THREATS = [
	{ id: "ALR-7821", subject: "URGENT: Wire transfer authorization required", sender: "cfo@acme-corp.tk", level: "critical" as const, score: 96, when: "2m ago" },
	{ id: "ALR-7820", subject: "Your Apple ID password was reset", sender: "no-reply@apple-id.support", level: "critical" as const, score: 91, when: "9m ago" },
	{ id: "ALR-7819", subject: "Invoice #1224 — overdue", sender: "billing@vendor-portal.app", level: "warning" as const, score: 78, when: "17m ago" },
	{ id: "ALR-7818", subject: "Shared file: Q3-Forecast.xlsx", sender: "alerts@dropbox-share.app", level: "warning" as const, score: 71, when: "24m ago" },
	{ id: "ALR-7817", subject: "Re: Vendor onboarding documents", sender: "ops@known-vendor.com", level: "caution" as const, score: 54, when: "38m ago" },
];

import { useEffect, useState } from "react";
import { Responsive, WidthProvider, Layout, Layouts } from "react-grid-layout";
import "react-grid-layout/css/styles.css";
import "react-resizable/css/styles.css";

const ResponsiveGridLayout = WidthProvider(Responsive);

const INITIAL_LAYOUTS: Layouts = {
	lg: [
		{ i: "timeline", x: 0, y: 0, w: 3, h: 1 },
		{ i: "severity", x: 3, y: 0, w: 1, h: 1 },
		{ i: "vector", x: 0, y: 1, w: 2, h: 1 },
		{ i: "senders", x: 2, y: 1, w: 2, h: 1 },
		{ i: "geomap", x: 0, y: 2, w: 3, h: 1 },
		{ i: "threats", x: 3, y: 2, w: 1, h: 1 },
	],
	md: [
		{ i: "timeline", x: 0, y: 0, w: 3, h: 1 },
		{ i: "severity", x: 0, y: 1, w: 3, h: 1 },
		{ i: "vector", x: 0, y: 2, w: 3, h: 1 },
		{ i: "senders", x: 0, y: 3, w: 3, h: 1 },
		{ i: "geomap", x: 0, y: 4, w: 3, h: 1 },
		{ i: "threats", x: 0, y: 5, w: 3, h: 1 },
	],
};

export default function DashboardPage() {
	const [isClient, setIsClient] = useState(false);
	const [isLocked, setIsLocked] = useState(true);
	const [layouts, setLayouts] = useState<Layouts>(INITIAL_LAYOUTS);

	useEffect(() => {
		setIsClient(true);
		const saved = localStorage.getItem("dashboard_layouts");
		if (saved) {
			try {
				setLayouts(JSON.parse(saved));
			} catch (e) {
				console.error("Failed to parse saved layout");
			}
		}
	}, []);

	const handleLayoutChange = (currentLayout: Layout[], allLayouts: Layouts) => {
		setLayouts(allLayouts);
		localStorage.setItem("dashboard_layouts", JSON.stringify(allLayouts));
	};

	return (
		<div className="flex flex-col gap-6 w-full max-w-7xl mx-auto pb-12">
			<div className="flex items-center justify-between">
				<div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 flex-1">
					<StatCard label="Inbound Today" value="2,847" change="+12.4%" trend="up" />
					<StatCard label="Threats Blocked" value="318" change="+8.2%" trend="up" />
					<StatCard label="Critical Alerts" value="14" change="+2" trend="up" />
					<StatCard label="Open Cases" value="9" change="-3" trend="down" />
					<StatCard label="Detection Rate" value="99.4%" change="+0.3%" trend="up" />
				</div>
				<div className="ml-6 flex-shrink-0 self-start">
					<Btn size="sm" variant={isLocked ? "default" : "accent"} onClick={() => setIsLocked(!isLocked)}>
						{isLocked ? "Unlock Layout" : "Lock Layout"}
					</Btn>
				</div>
			</div>

			<div className="mt-2 -mx-2">
				{isClient ? (
					<ResponsiveGridLayout
						className="layout"
						layouts={layouts}
						breakpoints={{ lg: 1200, md: 996, sm: 768, xs: 480, xxs: 0 }}
						cols={{ lg: 4, md: 3, sm: 2, xs: 1, xxs: 1 }}
						rowHeight={340}
						onLayoutChange={handleLayoutChange}
						isDraggable={!isLocked}
						isResizable={!isLocked}
						margin={[24, 24]}
					>
						<div key="timeline" className="h-full">
							<LineChartComponent data={TIMELINE} className="h-full" />
						</div>

						<div key="severity" className="h-full">
							<DonutChart data={SEVERITY_SEGMENTS} centerLabel="TOTAL" centerValue="300" className="h-full" />
						</div>

						<div key="vector" className="h-full">
							<RadarChart axes={VECTOR_AXES} className="h-full" />
						</div>

						<div key="senders" className="h-full">
							<BarChart data={TOP_SENDERS} className="h-full" />
						</div>

						<div key="geomap" className="h-full">
							<Card
								className="h-full flex flex-col"
								title="Geo Threat Origins"
								subtitle="Real-time origin map"
								actions={<span className="flex items-center gap-1.5 text-[11px] text-muted font-medium uppercase tracking-widest"><span className="w-2 h-2 rounded-full bg-success animate-pulse" /> LIVE</span>}
							>
								<div className="flex-1 min-h-0 relative rounded-md overflow-hidden bg-[#000000]">
									<InteractiveMap hotspots={HOTSPOTS} />
								</div>
							</Card>
						</div>

						<div key="threats" className="h-full">
							<Card
								className="h-full flex flex-col"
								title="Recent Threats"
								subtitle="Click to inspect"
								actions={<Btn size="sm">View All</Btn>}
							>
								<ul className="space-y-2 overflow-y-auto flex-1 min-h-0">
									{RECENT_THREATS.map((t) => (
										<li key={t.id} className="p-3 rounded-lg border border-transparent hover:border-border hover:bg-surface-2/50 transition-colors cursor-pointer">
											<div className="flex items-center justify-between gap-2 mb-1">
												<span className="text-[10px] font-mono text-muted">{t.id}</span>
												<Severity level={t.level} />
											</div>
											<div className="text-xs font-medium truncate text-foreground">{t.subject}</div>
											<div className="flex items-center justify-between gap-2 mt-2 text-[10px] text-muted">
												<span className="truncate text-secondary">{t.sender}</span>
												<span>{t.when}</span>
											</div>
										</li>
									))}
								</ul>
							</Card>
						</div>
					</ResponsiveGridLayout>
				) : (
					<div className="h-[1000px]" />
				)}
			</div>
		</div>
	);
}
