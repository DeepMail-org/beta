"use client";
import React, { useState, useEffect, useCallback, useRef } from "react";
import { Check, X, UploadCloud, File } from "lucide-react";
import { LiquidButton } from "./ui/liquid-glass-button";

export type ToastMessage = { id: string; message: string };

export function useToast() {
	const [toasts, setToasts] = useState<ToastMessage[]>([]);

	const removeToast = useCallback((id: string) => {
		setToasts((prev) => prev.filter((t) => t.id !== id));
	}, []);

	const addToast = useCallback(
		(message: string) => {
			const id = Math.random().toString(36).substr(2, 9);
			setToasts((prev) => [...prev, { id, message }]);
			setTimeout(() => {
				removeToast(id);
			}, 3500);
		},
		[removeToast],
	);

	return { toasts, addToast, removeToast };
}

export function ToastContainer({
	toasts,
	removeToast,
}: {
	toasts: ToastMessage[];
	removeToast: (id: string) => void;
}) {
	return (
		<div className="fixed bottom-6 right-6 z-50 flex flex-col gap-2 pointer-events-none">
			{toasts.map((t) => (
				<div
					key={t.id}
					className="glass-strong rounded-lg px-4 py-3 min-w-[260px] flex items-center justify-between pointer-events-auto transition-all animate-in slide-in-from-bottom-5 fade-in duration-300"
					style={{ animationFillMode: "forwards" }}
				>
					<div className="flex items-center gap-3">
						<div className="w-5 h-5 rounded-full bg-(--glass-strong) flex items-center justify-center shrink-0">
							<Check className="w-3 h-3 text-(--secondary)" />
						</div>
						<span className="text-[13.5px] font-body text-(--secondary)">
							{t.message}
						</span>
					</div>
					<button
						onClick={() => removeToast(t.id)}
						className="text-(--muted) hover:text-white transition-colors"
						aria-label="Close Toast"
					>
						<X className="w-4 h-4" />
					</button>
				</div>
			))}
		</div>
	);
}

export function UploadModal({
	isOpen,
	onClose,
	onToast,
}: {
	isOpen: boolean;
	onClose: () => void;
	onToast: (msg: string) => void;
}) {
	const [file, setFile] = useState<File | null>(null);
	const [analyzing, setAnalyzing] = useState(false);
	const [done, setDone] = useState(false);
	const fileInputRef = useRef<HTMLInputElement>(null);

	useEffect(() => {
		if (isOpen) {
			document.body.style.overflow = "hidden";
			setFile(null);
			setAnalyzing(false);
			setDone(false);
		} else {
			document.body.style.overflow = "";
		}
	}, [isOpen]);

	useEffect(() => {
		const handleEsc = (e: KeyboardEvent) => {
			if (e.key === "Escape") onClose();
		};
		window.addEventListener("keydown", handleEsc);
		return () => window.removeEventListener("keydown", handleEsc);
	}, [onClose]);

	if (!isOpen) return null;

	const handleDrop = (e: React.DragEvent) => {
		e.preventDefault();
		if (e.dataTransfer.files && e.dataTransfer.files[0]) {
			setFile(e.dataTransfer.files[0]);
		}
	};

	const handleAnalyze = () => {
		if (!file) return;
		setAnalyzing(true);
		setTimeout(() => {
			setAnalyzing(false);
			setDone(true);
			onToast("File analysis complete");
			setTimeout(() => onClose(), 1500);
		}, 2000);
	};

	return (
		<div className="fixed inset-0 bg-black/65 backdrop-blur-md z-50 flex items-center justify-center p-4">
			{/* Background overlay click to close */}
			<div
				className="absolute inset-0"
				onClick={onClose}
				aria-label="Close Modal overlay"
			></div>

			<div
				className="glass-strong glass-shine rounded-2xl p-8 max-w-[480px] w-full relative z-10 animate-in fade-in zoom-in-95 duration-200"
				role="dialog"
				aria-modal="true"
			>
				<button
					onClick={onClose}
					className="absolute top-6 right-6 text-(--muted) hover:text-white transition-colors"
				>
					<X className="w-5 h-5" />
				</button>

				<h2 className="font-display font-semibold text-[20px] text-(--foreground)">
					Analyze a Threat Email
				</h2>
				<p className="text-[13px] text-(--muted) mt-1 mb-6">
					Upload a suspicious .eml or .msg file securely for deep
					inspection.
				</p>

				<div
					className="border border-dashed border-white/10 hover:border-white/25 hover:bg-(--glass-strong) transition-colors rounded-xl p-10 text-center cursor-pointer mb-6"
					onDragOver={(e) => e.preventDefault()}
					onDrop={handleDrop}
					onClick={() => fileInputRef.current?.click()}
				>
					<input
						type="file"
						ref={fileInputRef}
						className="hidden"
						accept=".eml,.msg"
						onChange={(e) => setFile(e.target.files?.[0] || null)}
					/>
					{file ? (
						<div className="flex flex-col items-center gap-3">
							<div className="w-12 h-12 rounded-full bg-(--glass-strong) flex items-center justify-center">
								<Check className="w-6 h-6 text-white" />
							</div>
							<div>
								<p className="text-[14px] font-medium text-(--foreground)">
									{file.name}
								</p>
								<p className="text-[12px] text-(--muted)">
									{(file.size / 1024).toFixed(1)} KB
								</p>
							</div>
						</div>
					) : (
						<div className="flex flex-col items-center gap-3">
							<div className="w-12 h-12 rounded-full bg-(--glass) flex items-center justify-center text-(--muted)">
								<UploadCloud className="w-6 h-6" />
							</div>
							<p className="text-[14px] text-(--foreground) mt-2">
								Drop .eml or .msg file here
							</p>
							<p className="text-[13px] text-(--muted)">
								or{" "}
								<span className="underline decoration-white/20 underline-offset-4">
									browse files
								</span>
							</p>
						</div>
					)}
				</div>

				<LiquidButton
					onClick={handleAnalyze}
					disabled={!file || analyzing || done}
					className="w-full rounded-lg py-3 font-medium flex items-center justify-center gap-2 transition-all"
				>
					{analyzing ? (
						<>
							<div className="w-4 h-4 rounded-full border-2 border-[#05060A]/20 border-t-[#05060A] animate-spin" />{" "}
							Analyzing...
						</>
					) : done ? (
						<>
							<Check className="w-4 h-4" /> Results Ready
						</>
					) : (
						"Analyze"
					)}
				</LiquidButton>
			</div>
		</div>
	);
}

export function ContactModal({
	isOpen,
	onClose,
	onToast,
}: {
	isOpen: boolean;
	onClose: () => void;
	onToast: (msg: string) => void;
}) {
	const [submitting, setSubmitting] = useState(false);
	const [done, setDone] = useState(false);
	const [errors, setErrors] = useState<Record<string, boolean>>({});

	useEffect(() => {
		if (isOpen) {
			document.body.style.overflow = "hidden";
			setSubmitting(false);
			setDone(false);
			setErrors({});
		} else {
			document.body.style.overflow = "";
		}
	}, [isOpen]);

	if (!isOpen) return null;

	const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
		e.preventDefault();
		const formData = new FormData(e.currentTarget);
		const data = Object.fromEntries(formData);

		// Validate
		const newErrors: Record<string, boolean> = {};
		if (!data.name) newErrors.name = true;
		if (!data.email) newErrors.email = true;
		if (!data.company) newErrors.company = true;
		if (!data.message) newErrors.message = true;

		if (Object.keys(newErrors).length > 0) {
			setErrors(newErrors);
			return;
		}

		setSubmitting(true);
		setErrors({});
		setTimeout(() => {
			setSubmitting(false);
			setDone(true);
			onToast("Message sent successfully");
			setTimeout(() => onClose(), 2000);
		}, 1500);
	};

	const inputClass =
		"w-full bg-(--surface-2) border border-(--border) rounded-lg px-4 py-3 text-[14px] text-(--foreground) focus:outline-none focus:border-(--border-hover) transition-colors glass-shine";

	return (
		<div className="fixed inset-0 bg-black/65 backdrop-blur-md z-50 flex items-center justify-center p-4">
			{/* Background overlay click to close */}
			<div className="absolute inset-0" onClick={onClose}></div>

			<div
				className="glass-strong glass-shine rounded-2xl p-8 max-w-[480px] w-full relative z-10 animate-in fade-in zoom-in-95 duration-200"
				role="dialog"
				aria-modal="true"
			>
				<button
					onClick={onClose}
					disabled={submitting}
					className="absolute top-6 right-6 text-(--muted) hover:text-white transition-colors"
				>
					<X className="w-5 h-5" />
				</button>

				<h2 className="font-display font-semibold text-[20px] text-(--foreground) mb-6">
					Contact Sales
				</h2>

				{done ? (
					<div className="py-8 text-center flex flex-col items-center gap-4">
						<div className="w-12 h-12 rounded-full bg-(--glass-strong) border border-(--border) flex items-center justify-center">
							<Check className="w-6 h-6 text-white" />
						</div>
						<p className="text-[14px] text-(--secondary)">
							We&apos;ll get back to you... We&apos;ll respond
							within 24h.
						</p>
					</div>
				) : (
					<form
						className="flex flex-col gap-4"
						onSubmit={handleSubmit}
					>
						<div>
							<input
								name="name"
								type="text"
								placeholder="Name"
								className={`${inputClass} ${errors.name ? "border-white/15 bg-white/5" : ""}`}
							/>
						</div>
						<div>
							<input
								name="email"
								type="email"
								placeholder="Work Email"
								className={`${inputClass} ${errors.email ? "border-white/15 bg-white/5" : ""}`}
							/>
						</div>
						<div>
							<input
								name="company"
								type="text"
								placeholder="Company"
								className={`${inputClass} ${errors.company ? "border-white/15 bg-white/5" : ""}`}
							/>
						</div>
						<div>
							<textarea
								name="message"
								placeholder="Message"
								rows={4}
								className={`${inputClass} resize-none ${errors.message ? "border-white/15 bg-white/5" : ""}`}
							></textarea>
						</div>
						<LiquidButton
							type="submit"
							disabled={submitting}
							className="w-full mt-2 rounded-lg py-3 font-medium flex items-center justify-center gap-2 cursor-pointer transition-all"
						>
							{submitting ? (
								<>
									<div className="w-4 h-4 rounded-full border-2 border-[#05060A]/20 border-t-[#05060A] animate-spin" />{" "}
									Sending...
								</>
							) : (
								"Send Message"
							)}
						</LiquidButton>
					</form>
				)}
			</div>
		</div>
	);
}
