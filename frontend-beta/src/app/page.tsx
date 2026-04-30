"use client";
import { useState, useEffect, useRef } from "react";
import { LandingNavbar } from "@/components/LandingNavbar";
import { LandingHero } from "@/components/LandingHero";
import { LandingStats } from "@/components/LandingStats";
import { CinematicFooter } from "@/components/ui/motion-footer";
import { UploadModal } from "@/components/LandingModals";
import { useToast, ToastContainer } from "@/components/LandingModals";
import HoverBrandLogo from "@/components/ui/hover-brand-logo";
import FAQWithSpiral from "@/components/ui/faq-section";
import { EtheralShadow } from "@/components/ui/etheral-shadow";
import FeaturesSectionDemo2 from "@/components/features-section-demo-2";
import FeaturesSectionDemo3 from "@/components/features-section-demo-3";
import BentoGridThirdDemo from "@/components/bento-grid-demo-3";
import { FeaturesAndBenefits } from "@/components/features-and-benefits";

function PageSpotlight() {
	const [pos, setPos] = useState({ x: -400, y: -400 });
	const [active, setActive] = useState(false);

	useEffect(() => {
		const onMove = (e: MouseEvent) => {
			setPos({ x: e.clientX, y: e.clientY });
			if (!active) setActive(true);
		};
		window.addEventListener("mousemove", onMove, { passive: true });
		return () => window.removeEventListener("mousemove", onMove);
	}, [active]);

	return (
		<div
			className="pointer-events-none fixed inset-0 z-30 transition-opacity duration-500"
			style={{
				opacity: active ? 1 : 0,
				background: `radial-gradient(700px circle at ${pos.x}px ${pos.y}px, rgba(255,255,255,0.032), transparent 65%)`,
			}}
		/>
	);
}

function FadeIn({ children }: { children: React.ReactNode }) {
	const [isVisible, setIsVisible] = useState(false);
	const ref = useRef<HTMLDivElement>(null);

	useEffect(() => {
		const observer = new IntersectionObserver(
			([entry]) => {
				if (entry.isIntersecting) {
					setIsVisible(true);
					if (ref.current) observer.unobserve(ref.current);
				}
			},
			{ threshold: 0.1 },
		);

		if (ref.current) observer.observe(ref.current);
		return () => observer.disconnect();
	}, []);

	return (
		<div
			ref={ref}
			className={`transition-all duration-1000 ${isVisible ? "opacity-100 translate-y-0" : "opacity-0 translate-y-8"}`}
		>
			{children}
		</div>
	);
}

export default function DeepMailLandingPage() {
	const [uploadModalOpen, setUploadModalOpen] = useState(false);
	const { toasts, addToast, removeToast } = useToast();

	return (
		<div className="min-h-screen bg-(--background) selection:bg-white/20">
			<PageSpotlight />
			<LandingNavbar onUploadClick={() => setUploadModalOpen(true)} />

			<main>
				<LandingHero onUploadClick={() => setUploadModalOpen(true)} />

				<FadeIn>
					<HoverBrandLogo />
				</FadeIn>

				<FadeIn>
					<section className="relative h-120 w-full overflow-hidden border-y border-white/5 md:h-140">
						<EtheralShadow
							color="rgba(255, 255, 255, 0.55)"
							animation={{ scale: 100, speed: 90 }}
							noise={{ opacity: 0.6, scale: 1.2 }}
							sizing="fill"
						>
							<div className="flex flex-col items-center gap-5 px-6 text-center">
								<span className="rounded-full border border-white/15 bg-black/40 px-3 py-1 font-mono text-[10px] font-semibold uppercase tracking-[0.22em] text-white/65 backdrop-blur-sm">
									Threat intelligence
								</span>
								<h2 className="font-display text-5xl font-semibold tracking-tight text-white drop-shadow-[0_4px_24px_rgba(0,0,0,0.6)] md:text-7xl lg:text-8xl">
									Shadows,
									<br />
									surfaced.
								</h2>
								<p className="max-w-md text-sm leading-relaxed text-white/65 md:text-base">
									Every campaign leaves a trace. DeepMail finds it before your inbox does.
								</p>
							</div>
						</EtheralShadow>
					</section>
				</FadeIn>

				<FadeIn>
					<LandingStats />
				</FadeIn>

				<FadeIn>
					<section className="border-y border-white/5 py-12">
						<div className="mx-auto max-w-7xl px-6 mb-2 text-center">
							<p className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/40">
								Capabilities
							</p>
							<h2 className="mt-3 font-display text-3xl md:text-5xl font-semibold tracking-tight text-white">
								Everything a SOC needs in one pipeline
							</h2>
						</div>
						<FeaturesSectionDemo2 />
					</section>
				</FadeIn>

				<FadeIn>
					<FeaturesSectionDemo3 />
				</FadeIn>


				<FadeIn>
					<section className="py-20 border-t border-white/5">
						<div className="mx-auto mb-12 max-w-5xl px-6 text-center">
							<p className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/40">
								Inside the engine
							</p>
							<h2 className="mt-3 font-display text-3xl md:text-5xl font-semibold tracking-tight text-white">
								See how the pieces fit together
							</h2>
							<p className="mt-4 text-sm md:text-base text-white/40 max-w-xl mx-auto">
								Five interlocking layers turn a raw .eml into a verdict you can act on. Hover any tile.
							</p>
						</div>
						<BentoGridThirdDemo />
					</section>
				</FadeIn>

                <FadeIn>
					<FeaturesAndBenefits />
				</FadeIn>

				<FadeIn>
					<div className="py-12">
						<FAQWithSpiral />
					</div>
				</FadeIn>
			</main>

			<CinematicFooter />

			{uploadModalOpen && (
				<UploadModal
					isOpen={true}
					onClose={() => setUploadModalOpen(false)}
					onToast={addToast}
				/>
			)}

			<ToastContainer toasts={toasts} removeToast={removeToast} />
		</div>
	);
}
