export default function StatusFooter() {
	return (
		<footer className="mt-auto border-t border-outline-variant/10 bg-surface-container-low px-8 py-3 flex items-center justify-between text-[10px] font-bold tracking-widest text-on-surface-variant uppercase">
			<div className="flex items-center gap-6">
				<div className="flex items-center gap-2">
					<span className="w-2 h-2 bg-tertiary rounded-full" />
					<span>System Online</span>
				</div>
				<div className="flex items-center gap-2">
					<span className="w-2 h-2 bg-primary rounded-full animate-pulse" />
					<span>Engine: Active</span>
				</div>
				<div>CPU: 42%</div>
				<div>RAM: 12.4GB / 32GB</div>
			</div>
			<div>Build v4.22.09-Sentinel</div>
		</footer>
	);
}
