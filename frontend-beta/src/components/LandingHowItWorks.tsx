import { IconContainer, Radar } from "./ui/radar-effect";
import {
	FileText,
	Globe,
	Paperclip,
	Zap,
	AlertTriangle,
	Shield,
} from "lucide-react";

export function LandingHowItWorks() {
	return (
		<section className="relative flex flex-col items-center justify-center py-32 lg:py-48 w-full overflow-hidden min-h-[800px] bg-transparent">
			<div className="absolute z-21 pointer-events-none top-0 flex w-full flex-col items-center justify-start mt-20 md:mt-24 font-display">
				<h2 className="font-semibold text-4xl md:text-6xl tracking-tight text-white text-center px-4 drop-shadow-md">
					Real-Time Threat Pipeline
				</h2>
			</div>

			<div className="relative z-10 w-full max-w-5xl mx-auto px-6 h-[500px] md:h-[650px] flex items-center justify-center pointer-events-none mt-20 md:mt-32">
				<Radar className="absolute" />

				<div className="absolute left-1/2 top-1/2 -translate-x-48 -translate-y-40">
					<IconContainer
						icon={
							<FileText className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="PARSE"
						delay={0.1}
					/>
				</div>
				<div className="absolute left-1/2 top-1/2 translate-x-40 -translate-y-32">
					<IconContainer
						icon={
							<AlertTriangle className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="IOC TRIGGER"
						delay={0.2}
					/>
				</div>
				<div className="absolute left-1/2 top-1/2 translate-x-64 translate-y-16">
					<IconContainer
						icon={
							<Shield className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="GEO-INTEL"
						delay={0.3}
					/>
				</div>
				<div className="absolute left-1/2 top-1/2 -translate-x-56 translate-y-24">
					<IconContainer
						icon={
							<Globe className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="URL REG"
						delay={0.4}
					/>
				</div>
				<div className="absolute left-1/2 top-1/2 translate-x-24 translate-y-48">
					<IconContainer
						icon={
							<Paperclip className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="FILES"
						delay={0.5}
					/>
				</div>
				<div className="absolute left-1/2 top-1/2 -translate-x-80 -translate-y-8">
					<IconContainer
						icon={
							<Zap className="h-4 w-4 sm:h-5 sm:w-5 text-white" />
						}
						text="SCORING"
						delay={0.6}
					/>
				</div>
			</div>

			<div className="absolute bottom-0 inset-x-0 h-40 bg-linear-to-t from-(--background) via-(--background)/80 to-transparent z-10 pointer-events-none" />
			<div className="absolute top-0 inset-x-0 h-40 bg-linear-to-b from-(--background) via-(--background)/80 to-transparent z-10 pointer-events-none" />
		</section>
	);
}
