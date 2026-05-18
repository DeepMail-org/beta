"use client";

export function LandingStats() {
	const stats = [
		{ val: "<5ms", label: "Upload-to-queue" },
		{ val: "6-stage", label: "Analysis pipeline" },
		{ val: "3 providers", label: "Threat intel sources" },
		{ val: "100%", label: "Prepared statements" },
	];

	return (
		<div className="w-full bg-black py-32">
			<div className="max-w-280 mx-auto bg-white/6 grid sm:grid-cols-2 lg:grid-cols-4 gap-px">
				{stats.map((s, i) => (
					<div
						key={i}
						className="bg-black py-10 px-10 text-center flex flex-col items-center justify-center"
					>
						<div className="font-display font-semibold text-[44px] text-gradient tracking-[-0.03em] leading-none mb-2">
							{s.val}
						</div>
						<div className="font-body font-light text-[13px] text-(--muted)">
							{s.label}
						</div>
					</div>
				))}
			</div>
		</div>
	);
}
