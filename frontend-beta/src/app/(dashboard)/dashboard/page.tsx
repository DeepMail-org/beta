"use client";

import { useEffect } from "react";
import { Btn, StatCard } from "@/components/dashboard/primitives";
import { DashboardGrid } from "@/components/dashboard/DashboardGrid";
import { WidgetMarketplace } from "@/components/dashboard/WidgetMarketplace";
import { CommandPalette } from "@/components/dashboard/CommandPalette";
import { WidgetProvider } from "@/lib/dashboard/widget-context";
import { registerBuiltinWidgets } from "@/lib/dashboard/widgets";
import { useDashboardStore } from "@/stores/dashboard-store";
import { Mail, ShieldX, AlertTriangle, Briefcase, Activity } from "lucide-react";

registerBuiltinWidgets();

export default function DashboardPage() {
	const setMarketplaceOpen = useDashboardStore((s) => s.setMarketplaceOpen);

	return (
		<WidgetProvider>
			<div className="flex flex-col gap-6 w-full max-w-7xl mx-auto pb-12">
				<div className="flex items-center justify-between">
					<div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 flex-1">
						<StatCard label="Inbound Today" value="2,847" change="+12.4%" trend="up" icon={<Mail className="w-5 h-5" />} />
						<StatCard label="Threats Blocked" value="318" change="+8.2%" trend="up" icon={<ShieldX className="w-5 h-5" />} />
						<StatCard label="Critical Alerts" value="14" change="+2" trend="up" icon={<AlertTriangle className="w-5 h-5" />} />
						<StatCard label="Open Cases" value="9" change="-3" trend="down" icon={<Briefcase className="w-5 h-5" />} />
						<StatCard label="Detection Rate" value="99.4%" change="+0.3%" trend="up" icon={<Activity className="w-5 h-5" />} />
					</div>
					<div className="ml-6 shrink-0 self-start flex items-center gap-2">
						<Btn size="sm" onClick={() => setMarketplaceOpen(true)}>
							+ Add Widget
						</Btn>
					</div>
				</div>

				<div className="mt-2">
					<DashboardGrid />
				</div>
			</div>

			<WidgetMarketplace />
			<CommandPalette />
		</WidgetProvider>
	);
}
