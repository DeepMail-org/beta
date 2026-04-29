"use client";
import { useState, useEffect, useRef } from "react";
import { LandingNavbar } from "@/components/LandingNavbar";
import { LandingHero } from "@/components/LandingHero";
import { LandingStats } from "@/components/LandingStats";
import { CinematicFooter } from "@/components/ui/motion-footer";
import { UploadModal } from "@/components/LandingModals";
import { useToast, ToastContainer } from "@/components/LandingModals";
import { Features } from "@/components/blocks/features-8";
import HoverBrandLogo from "@/components/ui/hover-brand-logo";
import { PerspectiveMarquee } from "@/components/ui/remocn-perspective-marquee";
import { LandingHowItWorks } from "@/components/LandingHowItWorks";
import FAQWithSpiral from "@/components/ui/faq-section";

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
					<div className="py-20 border-y border-white/5 overflow-hidden">
						<PerspectiveMarquee />
					</div>
				</FadeIn>

				<FadeIn>
					<LandingStats />
				</FadeIn>

				<FadeIn>
					<Features />
				</FadeIn>

				<FadeIn>
					<div className="py-20">
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
