"use client";

import { useState } from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import { Mail, Phone, MapPin, ArrowUpRight } from "lucide-react";

const CONTACT_EMAIL = "contact@deepmail.ai";
const CONTACT_PHONE = "+91 1234567890";
const CONTACT_SUPPORT = "support@deepmail.ai";

export default function ContactPage() {
	const [form, setForm] = useState({
		name: "",
		email: "",
		company: "",
		message: "",
	});
	const [sent, setSent] = useState(false);
	const [sending, setSending] = useState(false);

	const set =
		(k: keyof typeof form) =>
		(e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) =>
			setForm({ ...form, [k]: e.target.value });

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		setSending(true);
		await new Promise((r) => setTimeout(r, 1200));
		setSending(false);
		setSent(true);
	};

	return (
		<div className="min-h-screen bg-[#0a0a0a] text-white">
			{/* Navbar */}
			<nav className="fixed top-0 inset-x-0 z-50 px-8 py-5 flex items-center justify-between bg-transparent">
				<Link href="/" className="flex items-center gap-2.5 group">
					<div className="w-8 h-8 rounded-lg bg-white/5 border border-white/10 flex items-center justify-center">
						<span className="font-display font-bold text-white text-[15px]">D</span>
					</div>
					<span className="font-display font-semibold text-[17px] text-white/90">DeepMail</span>
				</Link>
				<Link
					href="/"
					className="text-sm text-white/40 hover:text-white/80 transition-colors duration-200 flex items-center gap-1.5"
				>
					← Back to home
				</Link>
			</nav>

			{/* Main layout */}
			<div className="min-h-screen flex flex-col lg:flex-row">
				{/* LEFT — info + world map */}
				<motion.div
					initial={{ opacity: 0, x: -24 }}
					animate={{ opacity: 1, x: 0 }}
					transition={{ duration: 0.55, ease: "easeOut" }}
					className="flex flex-col justify-between lg:w-1/2 px-8 md:px-16 pt-32 pb-12"
				>
					<div>
						{/* Icon */}
						<div className="mb-8 inline-flex">
							<div
								className="w-14 h-14 rounded-2xl flex items-center justify-center"
								style={{
									background:
										"linear-gradient(135deg, #1e3a5f 0%, #0f2440 100%)",
									boxShadow:
										"0 0 0 1px rgba(255,255,255,0.06), 0 8px 32px rgba(0,0,0,0.5)",
								}}
							>
								<Mail className="h-6 w-6 text-[#5b9bd5]" strokeWidth={1.5} />
							</div>
						</div>

						{/* Heading */}
						<h1 className="font-display text-5xl md:text-6xl font-bold leading-tight mb-5">
							Contact us
						</h1>
						<p className="text-white/50 text-base leading-relaxed max-w-sm mb-8">
							We&apos;re always looking for ways to improve our products and
							services. Contact us and let us know how we can help you.
						</p>

						{/* Contact info row */}
						<div className="flex flex-wrap items-center gap-x-3 gap-y-2 text-sm text-white/40">
							<a
								href={`mailto:${CONTACT_EMAIL}`}
								className="hover:text-white/80 transition-colors duration-200"
							>
								{CONTACT_EMAIL}
							</a>
							<span className="text-white/20">•</span>
							<a
								href={`tel:${CONTACT_PHONE}`}
								className="hover:text-white/80 transition-colors duration-200"
							>
								{CONTACT_PHONE}
							</a>
							<span className="text-white/20">•</span>
							<a
								href={`mailto:${CONTACT_SUPPORT}`}
								className="hover:text-white/80 transition-colors duration-200"
							>
								{CONTACT_SUPPORT}
							</a>
						</div>
					</div>

					{/* World map */}
					<div className="mt-12 relative w-full max-w-lg">
						{/* Location pin */}
						<div className="absolute z-10 pointer-events-none" style={{ left: "38%", top: "36%" }}>
							<div className="relative">
								<div className="px-2.5 py-1 rounded-md bg-white/90 text-black text-[11px] font-semibold whitespace-nowrap shadow-lg">
									We are here
								</div>
								{/* Pin line */}
								<div className="absolute left-1/2 -translate-x-px w-px bg-white/60" style={{ height: "28px", top: "100%", boxShadow: "0 0 8px 2px rgba(100,180,255,0.5)" }} />
								{/* Ping dot */}
								<div className="absolute left-1/2 -translate-x-1/2" style={{ top: "calc(100% + 28px)" }}>
									<span className="relative flex h-2 w-2">
										<span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#5b9bd5] opacity-75" />
										<span className="relative inline-flex rounded-full h-2 w-2 bg-[#5b9bd5]" />
									</span>
								</div>
							</div>
						</div>
						{/* SVG world map */}
						{/* eslint-disable-next-line @next/next/no-img-element */}
						<img
							src="https://assets.aceternity.com/pro/world.svg"
							alt="World map"
							className="w-full opacity-30 select-none"
							draggable={false}
						/>
					</div>
				</motion.div>

				{/* RIGHT — form card */}
				<motion.div
					initial={{ opacity: 0, x: 24 }}
					animate={{ opacity: 1, x: 0 }}
					transition={{ duration: 0.55, ease: "easeOut", delay: 0.1 }}
					className="lg:w-1/2 flex items-center justify-center px-8 md:px-16 pt-24 pb-12 lg:pt-0"
				>
					{sent ? (
						<div className="w-full max-w-md flex flex-col items-center justify-center gap-5 py-24">
							<div
								className="w-16 h-16 rounded-2xl flex items-center justify-center"
								style={{ background: "linear-gradient(135deg, #1e3a5f, #0f2440)" }}
							>
								<ArrowUpRight className="h-7 w-7 text-[#5b9bd5]" />
							</div>
							<h2 className="font-display text-2xl font-semibold text-white">
								Message sent!
							</h2>
							<p className="text-white/40 text-sm text-center max-w-xs">
								Thanks for reaching out. We&apos;ll get back to you within 24
								hours.
							</p>
							<button
								onClick={() => { setSent(false); setForm({ name: "", email: "", company: "", message: "" }); }}
								className="mt-2 text-xs text-white/30 hover:text-white/60 transition-colors"
							>
								Send another message
							</button>
						</div>
					) : (
						<div
							className="w-full max-w-md rounded-2xl p-8"
							style={{
								background: "rgba(18,18,18,0.95)",
								border: "1px solid rgba(255,255,255,0.07)",
								boxShadow: "0 32px 80px rgba(0,0,0,0.7)",
							}}
						>
							<form onSubmit={handleSubmit} className="space-y-5">
								{/* Full name */}
								<div className="space-y-2">
									<label className="block text-sm font-medium text-white/70">
										Full name
									</label>
									<input
										type="text"
										value={form.name}
										onChange={set("name")}
										placeholder="Manu Arora"
										required
										className="w-full h-11 rounded-lg px-4 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200"
										style={{
											background: "rgba(255,255,255,0.04)",
											border: "1px solid rgba(255,255,255,0.08)",
										}}
										onFocus={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.20)"; e.currentTarget.style.background = "rgba(255,255,255,0.06)"; }}
										onBlur={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.08)"; e.currentTarget.style.background = "rgba(255,255,255,0.04)"; }}
									/>
								</div>

								{/* Email */}
								<div className="space-y-2">
									<label className="block text-sm font-medium text-white/70">
										Email Address
									</label>
									<input
										type="email"
										value={form.email}
										onChange={set("email")}
										placeholder="support@aceternity.com"
										required
										className="w-full h-11 rounded-lg px-4 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200"
										style={{
											background: "rgba(255,255,255,0.04)",
											border: "1px solid rgba(255,255,255,0.08)",
										}}
										onFocus={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.20)"; e.currentTarget.style.background = "rgba(255,255,255,0.06)"; }}
										onBlur={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.08)"; e.currentTarget.style.background = "rgba(255,255,255,0.04)"; }}
									/>
								</div>

								{/* Company */}
								<div className="space-y-2">
									<label className="block text-sm font-medium text-white/70">
										Company
									</label>
									<input
										type="text"
										value={form.company}
										onChange={set("company")}
										placeholder="Aceternity Labs LLC"
										className="w-full h-11 rounded-lg px-4 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200"
										style={{
											background: "rgba(255,255,255,0.04)",
											border: "1px solid rgba(255,255,255,0.08)",
										}}
										onFocus={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.20)"; e.currentTarget.style.background = "rgba(255,255,255,0.06)"; }}
										onBlur={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.08)"; e.currentTarget.style.background = "rgba(255,255,255,0.04)"; }}
									/>
								</div>

								{/* Message */}
								<div className="space-y-2">
									<label className="block text-sm font-medium text-white/70">
										Message
									</label>
									<textarea
										value={form.message}
										onChange={set("message")}
										placeholder="Type your message here"
										required
										rows={5}
										className="w-full rounded-lg px-4 py-3 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200 resize-none"
										style={{
											background: "rgba(255,255,255,0.04)",
											border: "1px solid rgba(255,255,255,0.08)",
										}}
										onFocus={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.20)"; e.currentTarget.style.background = "rgba(255,255,255,0.06)"; }}
										onBlur={(e) => { e.currentTarget.style.borderColor = "rgba(255,255,255,0.08)"; e.currentTarget.style.background = "rgba(255,255,255,0.04)"; }}
									/>
								</div>

								{/* Submit */}
								<button
									type="submit"
									disabled={sending}
									className="relative h-11 px-8 rounded-lg text-sm font-semibold text-white overflow-hidden group transition-all duration-300 disabled:opacity-60"
									style={{
										background: "rgba(255,255,255,0.08)",
										border: "1px solid rgba(255,255,255,0.12)",
									}}
									onMouseEnter={(e) => { e.currentTarget.style.background = "rgba(255,255,255,0.13)"; e.currentTarget.style.borderColor = "rgba(255,255,255,0.22)"; }}
									onMouseLeave={(e) => { e.currentTarget.style.background = "rgba(255,255,255,0.08)"; e.currentTarget.style.borderColor = "rgba(255,255,255,0.12)"; }}
								>
									{sending ? (
										<span className="flex items-center gap-2">
											<svg className="h-3.5 w-3.5 animate-spin" viewBox="0 0 24 24" fill="none">
												<circle className="opacity-25" cx="12" cy="12" r="10" stroke="white" strokeWidth="3" />
												<path className="opacity-75" fill="white" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
											</svg>
											Sending…
										</span>
									) : (
										"Submit"
									)}
								</button>
							</form>
						</div>
					)}
				</motion.div>
			</div>
		</div>
	);
}
