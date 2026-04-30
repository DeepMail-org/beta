"use client";

import { useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { motion } from "framer-motion";
import { ArrowUpRight, Mail, RefreshCw } from "lucide-react";
import {
	InputOTP,
	InputOTPGroup,
	InputOTPSeparator,
	InputOTPSlot,
} from "@/components/ui/interfaces-input-otp";

const streakStyle = (
	top: string,
	opacity: number,
	shadow: string,
): React.CSSProperties => ({
	position: "absolute",
	width: "220%",
	height: "1px",
	left: "-60%",
	top,
	transform: "rotate(-15.45deg)",
	transformOrigin: "center center",
	background: `rgba(255,255,255,${opacity})`,
	boxShadow: shadow,
});

export default function VerifyPage() {
	const router = useRouter();
	const [code, setCode] = useState("");
	const [isLoading, setIsLoading] = useState(false);
	const [error, setError] = useState("");
	const [resent, setResent] = useState(false);

	const handleVerify = async (e: React.FormEvent) => {
		e.preventDefault();
		if (code.length !== 6) {
			setError("Enter all 6 digits");
			return;
		}
		setError("");
		setIsLoading(true);
		await new Promise((r) => setTimeout(r, 1100));
		setIsLoading(false);
		// Success path — route to landing for now
		router.push("/");
	};

	const handleResend = async () => {
		setResent(true);
		setTimeout(() => setResent(false), 3000);
	};

	return (
		<div className="relative flex min-h-screen flex-col items-center justify-center overflow-hidden bg-black px-4 py-12">
			{/* Streaks + bokeh */}
			<div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden>
				<div
					style={streakStyle(
						"49%",
						0.75,
						"0 0 50px 14px rgba(255,255,255,0.16), 0 0 16px 3px rgba(255,255,255,0.28)",
					)}
				/>
				<div
					style={streakStyle(
						"65%",
						0.45,
						"0 0 35px 8px rgba(255,255,255,0.09), 0 0 10px 2px rgba(255,255,255,0.16)",
					)}
				/>
				<div
					className="absolute -left-48 top-1/4 h-130 w-130 rounded-full"
					style={{
						background:
							"radial-gradient(circle, rgba(255,255,255,0.045) 0%, transparent 70%)",
						filter: "blur(50px)",
					}}
				/>
				<div
					className="absolute -right-48 bottom-1/4 h-115 w-115 rounded-full"
					style={{
						background:
							"radial-gradient(circle, rgba(255,255,255,0.03) 0%, transparent 70%)",
						filter: "blur(50px)",
					}}
				/>
			</div>

			{/* Card halo blur — only blurs the area immediately around the card */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-0"
				style={{
					backdropFilter: "blur(14px)",
					WebkitBackdropFilter: "blur(14px)",
					maskImage:
						"radial-gradient(ellipse 360px 460px at center, black 0%, black 35%, transparent 80%)",
					WebkitMaskImage:
						"radial-gradient(ellipse 360px 460px at center, black 0%, black 35%, transparent 80%)",
				}}
			/>

			<motion.div
				initial={{ opacity: 0, y: 24 }}
				animate={{ opacity: 1, y: 0 }}
				transition={{ duration: 0.5, ease: "easeOut" }}
				className="relative z-10 w-full max-w-md"
			>
				{/* Logo */}
				<Link
					href="/"
					className="group mx-auto mb-10 flex w-fit items-center gap-2.5"
				>
					<div className="flex h-9 w-9 items-center justify-center rounded-xl border border-white/10 glass-strong transition-colors duration-200 group-hover:border-white/25">
						<span className="font-display text-[16px] font-bold text-white">D</span>
					</div>
					<span className="font-display text-[19px] font-semibold text-white/90">
						DeepMail
					</span>
				</Link>

				{/* Card */}
				<div className="glass relative overflow-hidden rounded-3xl border border-white/[0.07] p-8 shadow-[0_24px_80px_rgba(0,0,0,0.6)]">
					<div className="relative z-10">
						<div className="mb-6 flex items-center gap-3">
							<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
								<Mail className="h-5 w-5 text-white/60" strokeWidth={1.5} />
							</div>
							<div>
								<h1 className="font-display text-xl font-semibold text-white">
									Verify your email
								</h1>
								<p className="mt-0.5 text-xs text-white/35">
									We sent a 6-digit code to your inbox
								</p>
							</div>
						</div>

						<form onSubmit={handleVerify} className="space-y-6">
							<div className="space-y-2">
								<label className="text-xs font-medium uppercase tracking-wider text-white/45">
									Verification code
								</label>
								<div className="flex justify-center py-2">
									<InputOTP
										maxLength={6}
										value={code}
										onChange={(v) => {
											setCode(v);
											setError("");
										}}
										containerClassName="gap-2"
									>
										<InputOTPGroup className="gap-2">
											<InputOTPSlot
												index={0}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
											<InputOTPSlot
												index={1}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
											<InputOTPSlot
												index={2}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
										</InputOTPGroup>
										<InputOTPSeparator className="text-white/30" />
										<InputOTPGroup className="gap-2">
											<InputOTPSlot
												index={3}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
											<InputOTPSlot
												index={4}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
											<InputOTPSlot
												index={5}
												className="h-12 w-11 rounded-xl border border-white/10 bg-white/4 text-base font-semibold text-white shadow-none transition-all duration-200 first:rounded-l-xl last:rounded-r-xl data-[active=true]:border-white/35 data-[active=true]:bg-white/8 data-[active=true]:ring-2 data-[active=true]:ring-white/15"
											/>
										</InputOTPGroup>
									</InputOTP>
								</div>
								{error && (
									<p className="text-center text-xs text-rose-400">{error}</p>
								)}
							</div>

							<button
								type="submit"
								disabled={isLoading || code.length !== 6}
								className="liquid-glass group relative mt-2 h-12 w-full overflow-hidden rounded-full border border-white/20 text-[15px] font-medium text-white transition-all duration-300 hover:border-white/35 hover:bg-white/8 hover:shadow-[0_0_32px_rgba(255,255,255,0.16)] active:scale-[0.99] disabled:cursor-not-allowed disabled:opacity-60"
							>
								{isLoading ? (
									<span className="flex items-center justify-center gap-2">
										<svg className="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
											<circle
												className="opacity-25"
												cx="12"
												cy="12"
												r="10"
												stroke="white"
												strokeWidth="3"
											/>
											<path
												className="opacity-75"
												fill="white"
												d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
											/>
										</svg>
										Verifying…
									</span>
								) : (
									<span className="flex items-center justify-center gap-2">
										Verify & continue
										<ArrowUpRight className="h-4 w-4 transition-transform duration-300 group-hover:-translate-y-0.5 group-hover:translate-x-0.5" />
									</span>
								)}
							</button>
						</form>

						<div className="mt-6 flex items-center justify-center gap-2 text-xs text-white/35">
							<span>Didn&apos;t get it?</span>
							<button
								onClick={handleResend}
								className="inline-flex items-center gap-1 text-white/65 transition-colors duration-200 hover:text-white"
							>
								<RefreshCw className="h-3 w-3" />
								{resent ? "Sent again" : "Resend code"}
							</button>
						</div>
					</div>
				</div>

				<p className="mt-6 text-center text-xs text-white/18">
					Wrong account?{" "}
					<Link
						href="/login"
						className="font-medium text-white/55 transition-colors duration-200 hover:text-white"
					>
						Sign in instead
					</Link>
				</p>
			</motion.div>
		</div>
	);
}
