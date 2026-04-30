import { cn } from "@/lib/utils";
import {
	IconShieldLock,
	IconWorldWww,
	IconHash,
	IconLink,
	IconBolt,
	IconChartRadar,
	IconCode,
	IconBoxMultiple,
} from "@tabler/icons-react";

export default function FeaturesSectionDemo() {
	const features = [
		{
			title: "IOC Extraction",
			description:
				"Pull every IP, domain, URL and hash out of raw .eml in seconds — nothing gets missed.",
			icon: <IconShieldLock />,
		},
		{
			title: "Geo-Intelligence",
			description:
				"MaxMind-powered enrichment maps every signal to a country, ASN and reputation tier.",
			icon: <IconWorldWww />,
		},
		{
			title: "Hash Lookups",
			description:
				"Cross-check every attachment against VirusTotal, MalwareBazaar and our private corpus.",
			icon: <IconHash />,
		},
		{
			title: "URL Reputation",
			description:
				"Resolve, fingerprint and score every link — homoglyphs, redirects and freshly-minted domains included.",
			icon: <IconLink />,
		},
		{
			title: "Streaming Pipeline",
			description:
				"Redis Streams + WebSockets push live PARSE → SCORE updates while you watch.",
			icon: <IconBolt />,
		},
		{
			title: "Threat Scoring",
			description:
				"Six weighted stages collapse into a single 0–100 confidence rating you can trust.",
			icon: <IconChartRadar />,
		},
		{
			title: "Developer API",
			description:
				"REST + WebSocket endpoints for everything in the UI. Bring your own SIEM.",
			icon: <IconCode />,
		},
		{
			title: "Isolated Sandbox",
			description:
				"Every payload detonates inside an ephemeral, network-segmented worker. Zero retention.",
			icon: <IconBoxMultiple />,
		},
	];
	return (
		<div className="relative z-10 mx-auto grid max-w-7xl grid-cols-1 py-10 md:grid-cols-2 lg:grid-cols-4">
			{features.map((feature, index) => (
				<Feature key={feature.title} {...feature} index={index} />
			))}
		</div>
	);
}

const Feature = ({
	title,
	description,
	icon,
	index,
}: {
	title: string;
	description: string;
	icon: React.ReactNode;
	index: number;
}) => {
	return (
		<div
			className={cn(
				"group/feature relative flex flex-col py-10 lg:border-r border-white/6",
				(index === 0 || index === 4) && "lg:border-l border-white/6",
				index < 4 && "lg:border-b border-white/6",
			)}
		>
			{index < 4 && (
				<div className="pointer-events-none absolute inset-0 h-full w-full bg-linear-to-t from-white/4 to-transparent opacity-0 transition duration-200 group-hover/feature:opacity-100" />
			)}
			{index >= 4 && (
				<div className="pointer-events-none absolute inset-0 h-full w-full bg-linear-to-b from-white/4 to-transparent opacity-0 transition duration-200 group-hover/feature:opacity-100" />
			)}
			<div className="relative z-10 mb-4 px-10 text-white/45">{icon}</div>
			<div className="relative z-10 mb-2 px-10 text-lg font-bold">
				<div className="absolute left-0 inset-y-0 my-auto h-6 w-1 origin-center rounded-tr-full rounded-br-full bg-white/10 transition-all duration-200 group-hover/feature:h-8 group-hover/feature:bg-white/70" />
				<span className="inline-block text-white/90 transition duration-200 group-hover/feature:translate-x-2">
					{title}
				</span>
			</div>
			<p className="relative z-10 max-w-xs px-10 text-sm text-white/45 leading-relaxed">
				{description}
			</p>
		</div>
	);
};
