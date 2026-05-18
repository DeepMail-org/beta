"use client";

import { useState } from "react";
import Link from "next/link";
import { motion } from "framer-motion";
import { Mail, Phone, ArrowUpRight, Send, MessageSquare } from "lucide-react";
import { Input } from "@/components/ui/input";
import { LiquidButton } from "@/components/ui/liquid-glass-button";

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
		<div className="relative min-h-screen overflow-hidden bg-black text-white">
			{/* Ambient backdrop */}
			<div
				aria-hidden
				className="pointer-events-none absolute inset-0"
				style={{
					background:
						"radial-gradient(ellipse 70% 55% at 50% 0%, rgba(255,255,255,0.04) 0%, transparent 70%)",
				}}
			/>
			<div
				aria-hidden
				className="pointer-events-none absolute -left-48 top-1/3 h-120 w-120 rounded-full"
				style={{
					background:
						"radial-gradient(circle, rgba(91,155,213,0.06) 0%, transparent 70%)",
					filter: "blur(50px)",
				}}
			/>
			<div
				aria-hidden
				className="pointer-events-none absolute -right-48 bottom-1/4 h-105 w-105 rounded-full"
				style={{
					background:
						"radial-gradient(circle, rgba(255,255,255,0.03) 0%, transparent 70%)",
					filter: "blur(50px)",
				}}
			/>

			{/* Navbar */}
			<nav className="fixed top-0 inset-x-0 z-50 px-6 md:px-10 py-4 flex items-center justify-between bg-transparent">
				<Link href="/" className="flex items-center gap-2.5 group">
					<div className="flex h-8 w-8 items-center justify-center rounded-lg border border-white/10 bg-white/5">
						<span className="font-display font-bold text-white text-[15px]">D</span>
					</div>
					<span className="font-display font-semibold text-[17px] text-white/90">
						DeepMail
					</span>
				</Link>
				<Link
					href="/"
					className="flex items-center gap-1.5 text-sm text-white/40 transition-colors duration-200 hover:text-white/80"
				>
					← Back to home
				</Link>
			</nav>

			{/* Main */}
			<main className="relative mx-auto w-full max-w-7xl px-6 md:px-10 pt-32 pb-20 lg:pt-40 lg:pb-28">
				<div className="grid grid-cols-1 gap-12 lg:grid-cols-12 lg:gap-16">
					{/* LEFT — info column */}
					<motion.aside
						initial={{ opacity: 0, x: -24 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.55, ease: "easeOut" }}
						className="lg:col-span-5 flex flex-col gap-10"
					>
						<div>
							<div className="mb-7 inline-flex">
								<div className="flex h-12 w-12 items-center justify-center rounded-2xl border border-white/10 bg-white/5">
									<MessageSquare className="h-5 w-5 text-white/65" strokeWidth={1.5} />
								</div>
							</div>

							<h1 className="font-display text-4xl md:text-5xl lg:text-[56px] font-semibold leading-[1.05] tracking-tight">
								Talk to the
								<br />
								<span className="text-white/55">DeepMail team</span>
							</h1>

							<p className="mt-6 max-w-md text-base leading-relaxed text-white/45">
								Whether you&apos;re evaluating us for your SOC, integrating the
								API, or just curious about how we score threats — we&apos;d
								love to hear from you.
							</p>
						</div>

						{/* Contact info cards */}
						<div className="grid gap-3">
							<a
								href={`mailto:${CONTACT_EMAIL}`}
								className="group flex items-center gap-4 rounded-2xl border border-white/8 bg-white/2 p-4 transition-all duration-200 hover:border-white/20 hover:bg-white/4"
							>
								<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
									<Mail className="h-4 w-4 text-white/65" strokeWidth={1.5} />
								</div>
								<div className="flex-1 min-w-0">
									<p className="text-[11px] font-medium uppercase tracking-wider text-white/35">
										General
									</p>
									<p className="truncate text-sm font-medium text-white/85">
										{CONTACT_EMAIL}
									</p>
								</div>
								<ArrowUpRight className="h-4 w-4 text-white/30 transition-all duration-200 group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-white/80" />
							</a>

							<a
								href={`mailto:${CONTACT_SUPPORT}`}
								className="group flex items-center gap-4 rounded-2xl border border-white/8 bg-white/2 p-4 transition-all duration-200 hover:border-white/20 hover:bg-white/4"
							>
								<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
									<Mail className="h-4 w-4 text-white/65" strokeWidth={1.5} />
								</div>
								<div className="flex-1 min-w-0">
									<p className="text-[11px] font-medium uppercase tracking-wider text-white/35">
										Support
									</p>
									<p className="truncate text-sm font-medium text-white/85">
										{CONTACT_SUPPORT}
									</p>
								</div>
								<ArrowUpRight className="h-4 w-4 text-white/30 transition-all duration-200 group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-white/80" />
							</a>

							<a
								href={`tel:${CONTACT_PHONE}`}
								className="group flex items-center gap-4 rounded-2xl border border-white/8 bg-white/2 p-4 transition-all duration-200 hover:border-white/20 hover:bg-white/4"
							>
								<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
									<Phone className="h-4 w-4 text-white/65" strokeWidth={1.5} />
								</div>
								<div className="flex-1 min-w-0">
									<p className="text-[11px] font-medium uppercase tracking-wider text-white/35">
										Phone
									</p>
									<p className="truncate text-sm font-medium text-white/85">
										{CONTACT_PHONE}
									</p>
								</div>
								<ArrowUpRight className="h-4 w-4 text-white/30 transition-all duration-200 group-hover:-translate-y-0.5 group-hover:translate-x-0.5 group-hover:text-white/80" />
							</a>
						</div>

						{/* Footnote */}
						<div className="rounded-2xl border border-white/6 bg-white/1.5 p-5">
							<p className="text-xs uppercase tracking-wider text-white/35 font-medium mb-2">
								Response time
							</p>
							<p className="text-sm text-white/55 leading-relaxed">
								We respond to most inquiries within 24 hours. Critical incidents
								— page our SOC line.
							</p>
						</div>

						{/* Book a call CTA */}
						<div className="flex flex-col gap-3 rounded-2xl border border-white/6 bg-white/1.5 p-5">
							<p className="text-xs uppercase tracking-wider text-white/35 font-medium">
								Prefer a call?
							</p>
							<p className="text-sm text-white/55 leading-relaxed">
								Schedule a 30-minute intro call with our team directly.
							</p>
							<Link href="https://cal.com" target="_blank" rel="noopener noreferrer" className="w-full">
								<LiquidButton size="sm" className="w-full rounded-full px-4 py-2 text-xs font-medium text-white whitespace-nowrap">
									Book a call
									<ArrowUpRight className="h-3.5 w-3.5 shrink-0" strokeWidth={1.5} />
								</LiquidButton>
							</Link>
						</div>
					</motion.aside>

					{/* RIGHT — form */}
					<motion.div
						initial={{ opacity: 0, x: 24 }}
						animate={{ opacity: 1, x: 0 }}
						transition={{ duration: 0.55, ease: "easeOut", delay: 0.1 }}
						className="lg:col-span-7"
					>
						{sent ? (
							<div className="flex flex-col items-center justify-center gap-5 rounded-3xl border border-white/8 bg-white/2 px-6 py-20">
								<div className="flex h-16 w-16 items-center justify-center rounded-2xl border border-white/12 bg-white/5">
									<Send className="h-6 w-6 text-emerald-400" strokeWidth={1.5} />
								</div>
								<h2 className="font-display text-2xl font-semibold text-white">
									Message sent
								</h2>
								<p className="max-w-xs text-center text-sm text-white/45">
									Thanks for reaching out — we&apos;ll get back to you within 24 hours.
								</p>
								<button
									onClick={() => {
										setSent(false);
										setForm({ name: "", email: "", company: "", message: "" });
									}}
									className="mt-2 text-xs text-white/35 transition-colors hover:text-white/70"
								>
									Send another message
								</button>
							</div>
						) : (
							<div className="rounded-3xl border border-white/8 bg-white/2 p-6 md:p-10 shadow-[0_24px_80px_rgba(0,0,0,0.5)]">
								<div className="mb-8">
									<h2 className="font-display text-2xl font-semibold text-white">
										Send us a message
									</h2>
									<p className="mt-2 text-sm text-white/40">
										We&apos;ll route it to the right team — analyst, sales or engineering.
									</p>
								</div>

								<form onSubmit={handleSubmit} className="space-y-5">
									<div className="grid grid-cols-1 gap-5 md:grid-cols-2">
										<div className="space-y-2">
											<label className="block text-xs font-medium uppercase tracking-wider text-white/45">
												Full name
											</label>
											<Input
												type="text"
												value={form.name}
												onChange={set("name")}
												placeholder="Jane Smith"
												required
											/>
										</div>

										<div className="space-y-2">
											<label className="block text-xs font-medium uppercase tracking-wider text-white/45">
												Email
											</label>
											<Input
												type="email"
												value={form.email}
												onChange={set("email")}
												placeholder="you@company.com"
												required
											/>
										</div>
									</div>

									<div className="space-y-2">
										<label className="block text-xs font-medium uppercase tracking-wider text-white/45">
											Company
										</label>
										<Input
											type="text"
											value={form.company}
											onChange={set("company")}
											placeholder="Acme Security"
										/>
									</div>

									<div className="space-y-2">
										<label className="block text-xs font-medium uppercase tracking-wider text-white/45">
											Message
										</label>
										<textarea
											value={form.message}
											onChange={set("message")}
											placeholder="What are you working on?"
											required
											rows={6}
											className="w-full resize-none rounded-2xl border border-white/10 bg-white/2 px-4 py-3 text-sm text-white placeholder:text-white/25 outline-none transition-all duration-200 hover:border-white/20 focus:border-white/30 focus:bg-white/5"
										/>
									</div>

									<div className="flex items-center justify-between gap-4 pt-2 flex-wrap">
										<p className="text-xs text-white/30">
											By submitting, you agree to our terms.
										</p>
										<LiquidButton
											type="submit"
											disabled={sending}
											size="lg"
											className="h-11 rounded-full px-6 py-2 text-sm font-medium text-white whitespace-nowrap"
										>
											{sending ? (
												<>
													<svg className="h-3.5 w-3.5 animate-spin shrink-0" viewBox="0 0 24 24" fill="none">
														<circle className="opacity-25" cx="12" cy="12" r="10" stroke="white" strokeWidth="3" />
														<path className="opacity-75" fill="white" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
													</svg>
													Sending…
												</>
											) : (
												<>
													<Send className="h-3.5 w-3.5 shrink-0" strokeWidth={1.5} />
													Send message
												</>
											)}
										</LiquidButton>
									</div>
								</form>
							</div>
						)}
					</motion.div>
				</div>
			</main>
		</div>
	);
}
