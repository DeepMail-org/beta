"use client";
import { useState, useEffect, useRef, memo } from "react";
import dynamic from "next/dynamic";
import { LandingNavbar } from "@/components/LandingNavbar";
import { LandingHero } from "@/components/LandingHero";
import { useToast, ToastContainer } from "@/components/LandingModals";

// Heavy below-the-fold sections — lazy-loaded on scroll. Each is rendered inside
// a FadeIn so loading happens just before it enters the viewport.
const LandingStats = dynamic(
	() => import("@/components/LandingStats").then((m) => m.LandingStats),
	{ loading: () => <SectionSkeleton h="h-72" />, ssr: false },
);
const HoverBrandLogo = dynamic(
	() => import("@/components/ui/hover-brand-logo"),
	{ loading: () => <SectionSkeleton h="h-64" />, ssr: false },
);
const FeaturesSectionDemo2 = dynamic(
	() => import("@/components/features"),
	{ loading: () => <SectionSkeleton h="h-[640px]" />, ssr: false },
);
const FeaturesSectionDemo3 = dynamic(
	() => import("@/components/features_section"),
	{ loading: () => <SectionSkeleton h="h-[720px]" />, ssr: false },
);
const BentoGridThirdDemo = dynamic(
	() => import("@/components/bento-grid"),
	{ loading: () => <SectionSkeleton h="h-[720px]" />, ssr: false },
);
const FeaturesAndBenefits = dynamic(
	() => import("@/components/features-and-benefits").then((m) => m.FeaturesAndBenefits),
	{ loading: () => <SectionSkeleton h="h-[480px]" />, ssr: false },
);
const PerspectiveMarquee = dynamic(
	() => import("@/components/ui/remocn-perspective-marquee").then((m) => m.PerspectiveMarquee),
	{ loading: () => <SectionSkeleton h="h-36" />, ssr: false },
);
const CallToAction = dynamic(
	() => import("@/components/ui/cta-3").then((m) => m.CallToAction),
	{ loading: () => <SectionSkeleton h="h-48" />, ssr: false },
);
const FAQWithSpiral = dynamic(
	() => import("@/components/ui/faq-section"),
	{ loading: () => <SectionSkeleton h="h-[640px]" />, ssr: false },
);
const CinematicFooter = dynamic(
	() => import("@/components/ui/motion-footer").then((m) => m.CinematicFooter),
	{ loading: () => null, ssr: false },
);
const UploadModal = dynamic(
	() => import("@/components/LandingModals").then((m) => m.UploadModal),
	{ ssr: false },
);

function SectionSkeleton({ h = "h-96" }: { h?: string }) {
	return (
		<div className={`w-full ${h} animate-pulse bg-white/1.5`} aria-hidden />
	);
}

const PageSpotlight = memo(function PageSpotlight() {
	const [pos, setPos] = useState({ x: -400, y: -400 });
	const [active, setActive] = useState(false);

	useEffect(() => {
		let raf = 0;
		let pending: { x: number; y: number } | null = null;
		const onMove = (e: MouseEvent) => {
			pending = { x: e.clientX, y: e.clientY };
			if (raf) return;
			raf = requestAnimationFrame(() => {
				if (pending) setPos(pending);
				if (!active) setActive(true);
				raf = 0;
			});
		};
		window.addEventListener("mousemove", onMove, { passive: true });
		return () => {
			window.removeEventListener("mousemove", onMove);
			if (raf) cancelAnimationFrame(raf);
		};
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
});

function FadeIn({ children }: { children: React.ReactNode }) {
	const [isVisible, setIsVisible] = useState(false);
	const ref = useRef<HTMLDivElement>(null);

	useEffect(() => {
		const node = ref.current;
		if (!node) return;
		const observer = new IntersectionObserver(
			([entry]) => {
				if (entry.isIntersecting) {
					setIsVisible(true);
					observer.unobserve(node);
				}
			},
			{ threshold: 0.1, rootMargin: "200px 0px" },
		);
		observer.observe(node);
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
					<LandingStats />
				</FadeIn>

				<FadeIn>
					<div className="relative h-48 w-full overflow-hidden border-y border-white/5">
						<PerspectiveMarquee
							fontSize={72}
							color="#fafafa"
							pixelsPerFrame={1.5}
							rotateY={-28}
							rotateX={8}
							perspective={1200}
							fadeColor="#000000"
							background="#000000"
						/>
					</div>
				</FadeIn>

				<FadeIn>
					<section className="border-y border-white/5 py-12">
						<div className="mx-auto mb-2 max-w-7xl px-6 text-center">
							<p className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/40">
								Capabilities
							</p>
							<h2 className="mt-3 font-display text-3xl font-semibold tracking-tight text-white md:text-5xl">
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
					<section className="border-t border-white/5 py-20">
						<div className="mx-auto mb-12 max-w-5xl px-6 text-center">
							<p className="font-mono text-[10px] font-semibold uppercase tracking-[0.18em] text-white/40">
								Inside the engine
							</p>
							<h2 className="mt-3 font-display text-3xl font-semibold tracking-tight text-white md:text-5xl">
								See how the pieces fit together
							</h2>
							<p className="mx-auto mt-4 max-w-xl text-sm text-white/40 md:text-base">
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
					<div className="py-20 px-6">
						<CallToAction />
					</div>
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
					isOpen
					onClose={() => setUploadModalOpen(false)}
					onToast={addToast}
				/>
			)}

			<ToastContainer toasts={toasts} removeToast={removeToast} />
		</div>
	);
}
