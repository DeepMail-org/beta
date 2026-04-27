export default function ReportsPage() {
	const dummyReports = [
		{
			id: "REP-198",
			title: "Quarterly Threat Landscape",
			date: "2024-03-28",
			type: "Executive",
			items: 450,
		},
		{
			id: "REP-197",
			title: "Phishing Campaign Analysis",
			date: "2024-03-25",
			type: "Technical",
			items: 12,
		},
		{
			id: "REP-196",
			title: "Weekly IOC Export",
			date: "2024-03-21",
			type: "Data Dump",
			items: 1840,
		},
		{
			id: "REP-195",
			title: "Zero-Day Heuristics Review",
			date: "2024-03-15",
			type: "Internal",
			items: 3,
		},
	];

	return (
		<div className="p-8 lg:p-12 space-y-12 max-w-7xl mx-auto">
			<div className="flex items-center justify-between">
				<div>
					<h2 className="text-4xl font-bold text-on-surface tracking-tight font-headline">
						THREAT_REPORTS
					</h2>
					<p className="text-on-surface-variant mt-2 text-sm">
						Downloadable executive and technical intelligence
						reports.
					</p>
				</div>
				<button className="px-4 py-2 bg-primary text-on-primary font-bold text-xs uppercase tracking-widest rounded flex items-center gap-2 hover:bg-primary/90 transition-colors">
					<span className="material-symbols-outlined text-[16px]">
						add
					</span>
					Generate Report
				</button>
			</div>

			<div className="glass-panel rounded-xl overflow-hidden">
				<table className="w-full text-left">
					<thead className="text-[10px] uppercase tracking-widest text-outline bg-bg/40">
						<tr>
							<th className="px-8 py-4 font-bold">
								Report ID / Title
							</th>
							<th className="px-8 py-4 font-bold">
								Date Generated
							</th>
							<th className="px-8 py-4 font-bold">Type</th>
							<th className="px-8 py-4 font-bold">Data Points</th>
							<th className="px-8 py-4 font-bold">Action</th>
						</tr>
					</thead>
					<tbody className="text-sm">
						{dummyReports.map((report) => (
							<tr
								key={report.id}
								className="hover:bg-surface-container-high/40 transition-colors border-b border-outline-variant/10"
							>
								<td className="px-8 py-4">
									<p className="font-bold text-on-surface">
										{report.title}
									</p>
									<p className="text-[10px] text-outline font-mono mt-0.5">
										{report.id}
									</p>
								</td>
								<td className="px-8 py-4 text-outline font-mono text-xs">
									{report.date}
								</td>
								<td className="px-8 py-4">
									<span className="bg-primary-container/10 text-primary-container text-[10px] px-2 py-1 rounded border border-primary-container/20 font-bold uppercase">
										{report.type}
									</span>
								</td>
								<td className="px-8 py-4 text-on-surface-variant font-mono">
									{report.items}
								</td>
								<td className="px-8 py-4">
									<button className="text-primary hover:text-primary/80 flex items-center gap-1 text-[10px] font-bold uppercase tracking-widest">
										<span className="material-symbols-outlined text-[16px]">
											download
										</span>
										Export PDF
									</button>
								</td>
							</tr>
						))}
					</tbody>
				</table>
			</div>
		</div>
	);
}
