"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { motion } from "framer-motion";
import { Eye, EyeOff, ArrowUpRight, Zap } from "lucide-react";
import { Input } from "@/components/ui/input";
import { FloatingPaths } from "@/components/ui/background-paths";
import { LiquidButton } from "@/components/ui/liquid-glass-button";

export default function SignupPage() {
	const router = useRouter();
	const [form, setForm] = useState({ name: "", email: "", password: "" });
	const [showPassword, setShowPassword] = useState(false);
	const [isLoading, setIsLoading] = useState(false);
	const [spot, setSpot] = useState({ x: 0, y: 0 });
	const [cardHovered, setCardHovered] = useState(false);

	const set = (key: keyof typeof form) => (e: React.ChangeEvent<HTMLInputElement>) =>
		setForm({ ...form, [key]: e.target.value });

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setIsLoading(true);
		await new Promise((r) => setTimeout(r, 1200));
		setIsLoading(false);
		router.push("/verify");
	};

	const onCardMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
		const r = e.currentTarget.getBoundingClientRect();
		setSpot({ x: e.clientX - r.left, y: e.clientY - r.top });
	};

	return (
		<div className="min-h-screen bg-black flex flex-col items-center justify-center px-4 py-12 relative overflow-hidden">
			{/* Background — animated paths */}
			<div className="absolute inset-0 pointer-events-none" aria-hidden>
				<FloatingPaths position={1} />
				<FloatingPaths position={-1} />
			</div>

			{/* Card halo blur — only blurs the area immediately around the card */}
			<div
				className="pointer-events-none absolute inset-0"
				style={{
					backdropFilter: "blur(14px)",
					WebkitBackdropFilter: "blur(14px)",
					maskImage:
						"radial-gradient(ellipse 360px 520px at center, black 0%, black 38%, transparent 82%)",
					WebkitMaskImage:
						"radial-gradient(ellipse 360px 520px at center, black 0%, black 38%, transparent 82%)",
				}}
				aria-hidden
			/>

			<motion.div
				initial={{ opacity: 0, y: 24 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, ease: "easeOut" }}
				className="relative z-10 w-full max-w-md"
			>
				{/* Logo */}
				<Link href="/" className="flex items-center gap-2.5 mb-10 w-fit mx-auto group">
					<div className="w-9 h-9 rounded-xl glass-strong flex items-center justify-center border border-white/10 group-hover:border-white/25 transition-colors duration-200">
						<span className="font-display font-bold text-white text-[16px]">D</span>
					</div>
					<span className="font-display font-semibold text-[19px] text-white/90">DeepMail</span>
				</Link>

				{/* Card with mouse-move spotlight */}
				<div
					className="rounded-[2rem] p-8 border border-white/10 bg-white/[0.04] shadow-[0_24px_80px_rgba(0,0,0,0.6)] relative overflow-hidden backdrop-blur-xl"
					onMouseMove={onCardMouseMove}
					onMouseEnter={() => setCardHovered(true)}
					onMouseLeave={() => setCardHovered(false)}
				>
					{/* Spotlight follows cursor inside card */}
					<div
						className="pointer-events-none absolute inset-0 rounded-[2rem] transition-opacity duration-300"
						style={{
							opacity: cardHovered ? 1 : 0,
							background: `radial-gradient(350px circle at ${spot.x}px ${spot.y}px, rgba(255,255,255,0.07), transparent 60%)`,
						}}
					/>

					<div className="relative z-10">
						<div className="flex items-center gap-3 mb-6">
							<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
								<Zap className="h-5 w-5 text-white/60" strokeWidth={1.5} />
							</div>
							<div>
								<h1 className="font-display text-xl font-semibold text-white">Create account</h1>
								<p className="text-xs text-white/35 mt-0.5">Start analyzing threats today</p>
							</div>
						</div>

						<form onSubmit={handleSubmit} className="space-y-4">
							<div className="space-y-1.5">
								<label className="text-xs font-medium text-white/45 uppercase tracking-wider">Full Name</label>
								<Input
									type="text"
									value={form.name}
									onChange={set("name")}
									placeholder="Jane Smith"
									required
								/>
							</div>

							<div className="space-y-1.5">
								<label className="text-xs font-medium text-white/45 uppercase tracking-wider">Email</label>
								<Input
									type="email"
									value={form.email}
									onChange={set("email")}
									placeholder="you@example.com"
									required
								/>
							</div>

							<div className="space-y-1.5">
								<label className="text-xs font-medium text-white/45 uppercase tracking-wider">Password</label>
								<div className="relative">
									<Input
										type={showPassword ? "text" : "password"}
										value={form.password}
										onChange={set("password")}
										placeholder="Min. 8 characters"
										required
										minLength={8}
										className="pr-11"
									/>
									<button
										type="button"
										onClick={() => setShowPassword(!showPassword)}
										className="absolute right-3 top-1/2 -translate-y-1/2 text-white/30 hover:text-white/65 transition-colors duration-200"
									>
										{showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
									</button>
								</div>
							</div>

							<LiquidButton
								type="submit"
								disabled={isLoading}
								className="w-full h-12 rounded-full text-[15px] font-medium text-white mt-2"
							>
								{isLoading ? (
									<span className="flex items-center justify-center gap-2">
										<svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
											<circle className="opacity-25" cx="12" cy="12" r="10" stroke="white" strokeWidth="3" />
											<path className="opacity-75" fill="white" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
										</svg>
										Creating account…
									</span>
								) : (
									<span className="flex items-center justify-center gap-2">
										Get Started Free
										<ArrowUpRight className="h-4 w-4" />
									</span>
								)}
							</LiquidButton>
						</form>

						<p className="text-center text-xs text-white/25 mt-6">
							Already have an account?{" "}
							<Link href="/login" className="text-white/55 hover:text-white transition-colors duration-200 font-medium">
								Sign in
							</Link>
						</p>
					</div>
				</div>

				<p className="text-center text-xs text-white/18 mt-6">
					By signing up you agree to our Terms &amp; Privacy Policy
				</p>
			</motion.div>
		</div>
	);
}
