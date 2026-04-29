import { useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import {
	SiRust,
	SiReact,
	SiRedis,
	SiCloudflare,
	SiDocker,
    SiVirustotal,
    SiBun,
    SiCodesandbox,
    SiKalilinux,
    SiKubernetes,
} from "react-icons/si";

const brands = [
	{ id: "bun", name: "Frontend API", Icon: SiBun },
	{ id: "react", name: "React", Icon: SiReact },
	{ id: "redis", name: "Redis", Icon: SiRedis },
	{ id: "rust", name: "Rust", Icon: SiRust },
	{ id: "vt", name: "Virus Total", Icon: SiVirustotal },
	{ id: "kali", name: "Kali Linux", Icon: SiKalilinux },
	{ id: "aws", name: "Cloudflare", Icon: SiCloudflare },
	{ id: "sandbox", name: "Sandbox", Icon: SiCodesandbox },
	{ id: "docker", name: "Docker", Icon: SiDocker },
	{ id: "kubernetes", name: "Kubernetes", Icon: SiKubernetes },
];

export default function HoverBrandLogo() {
	const [hoveredId, setHoveredId] = useState<string | null>(null);
	const activeBrand = brands.find((b) => b.id === hoveredId);

	return (
		<div className="w-full max-w-5xl mx-auto py-8 sm:py-16 px-4">
			<div className="flex flex-col sm:flex-row items-center justify-between gap-8 sm:gap-12 lg:gap-24">
				{/* Left: text */}
				<div className="shrink-0 w-full sm:w-auto text-center sm:text-left">
					<p className="text-sm sm:text-base text-muted-foreground font-medium mb-0 tracking-tight">
						Powered By
					</p>
					<div className="relative">
						<p
							aria-hidden
							className="text-3xl lg:text-3xl font-bold tracking-tight whitespace-nowrap opacity-0 pointer-events-none select-none leading-none sm:leading-tight"
						>
							Deep Integrations
						</p>
						<div className="absolute inset-0 overflow-hidden">
							<AnimatePresence mode="wait">
								<motion.p
									key={hoveredId ?? "default"}
									initial={{ y: 16, opacity: 0 }}
									animate={{ y: 0, opacity: 1 }}
									exit={{ y: -16, opacity: 0 }}
									transition={{
										duration: 0.16,
										ease: [0.25, 0.46, 0.45, 0.94],
									}}
									className="text-xl sm:text-2xl lg:text-3xl font-bold text-foreground leading-none sm:leading-tight tracking-tight whitespace-nowrap"
								>
									{activeBrand?.name ?? "Deep Integrations"}
								</motion.p>
							</AnimatePresence>
						</div>
					</div>
				</div>

				{/* Right: icon grid */}
				<div className="grid grid-cols-4 sm:flex sm:flex-wrap items-center justify-center sm:justify-end gap-1.5 sm:gap-2 w-full sm:w-auto md:mt-6 sm:mt-0">
					{brands.map(({ id, name, Icon }) => {
						const isActive = hoveredId === id;
						const isDimmed = hoveredId !== null && !isActive;
						return (
							<button
								key={id}
								aria-label={name}
								className={[
									"flex items-center justify-center p-2.5 sm:p-3 lg:p-3.5 rounded-lg border transition-all duration-200",
									isActive
										? "border-foreground/30 text-foreground bg-foreground/5"
										: "border-transparent text-foreground/30 hover:text-foreground/50",
									isDimmed ? "opacity-40 " : "",
								].join(" ")}
								onMouseEnter={() => setHoveredId(id)}
								onMouseLeave={() => setHoveredId(null)}
							>
								<Icon className="w-8 h-8 sm:w-6 sm:h-6" />
							</button>
						);
					})}
				</div>
			</div>
		</div>
	);
}
