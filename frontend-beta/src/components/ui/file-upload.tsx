"use client";

import React, { useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Upload, File, X, CheckCircle } from "lucide-react";
import { cn } from "@/lib/utils";

interface FileUploadProps {
	onChange?: (files: File[]) => void;
	accept?: string;
	maxSize?: number;
}

export function FileUpload({
	onChange,
	accept = ".eml,message/rfc822",
	maxSize = 25,
}: FileUploadProps) {
	const [isDragging, setIsDragging] = useState(false);
	const [files, setFiles] = useState<File[]>([]);
	const [error, setError] = useState<string | null>(null);
	const inputRef = useRef<HTMLInputElement>(null);

	const validateAndAdd = (incoming: File[]) => {
		setError(null);
		const oversized = incoming.find((f) => f.size > maxSize * 1024 * 1024);
		if (oversized) {
			setError(`File exceeds ${maxSize}MB limit`);
			return;
		}
		const next = [...files, ...incoming];
		setFiles(next);
		onChange?.(next);
	};

	const removeFile = (idx: number) => {
		const next = files.filter((_, i) => i !== idx);
		setFiles(next);
		onChange?.(next);
	};

	const onDrop = (e: React.DragEvent) => {
		e.preventDefault();
		setIsDragging(false);
		validateAndAdd(Array.from(e.dataTransfer.files));
	};

	const onInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		if (e.target.files) validateAndAdd(Array.from(e.target.files));
		e.target.value = "";
	};

	const formatSize = (bytes: number) => {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	};

	return (
		<div className="w-full space-y-3">
			{/* Drop zone */}
			<div
				onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
				onDragLeave={() => setIsDragging(false)}
				onDrop={onDrop}
				onClick={() => inputRef.current?.click()}
				className={cn(
					"relative cursor-pointer rounded-2xl border-2 border-dashed transition-all duration-300 group",
					isDragging
						? "border-white/60 bg-white/5 scale-[1.01]"
						: "border-white/15 bg-white/[0.02] hover:border-white/30 hover:bg-white/[0.04]",
				)}
			>
				<input
					ref={inputRef}
					type="file"
					accept={accept}
					multiple
					className="hidden"
					onChange={onInputChange}
				/>

				{/* Gradient corner accents */}
				<div className="pointer-events-none absolute -inset-px rounded-2xl overflow-hidden">
					<div className="absolute top-0 left-0 w-24 h-24 bg-[radial-gradient(circle_at_top_left,rgba(255,255,255,0.06),transparent_70%)]" />
					<div className="absolute bottom-0 right-0 w-24 h-24 bg-[radial-gradient(circle_at_bottom_right,rgba(255,255,255,0.04),transparent_70%)]" />
				</div>

				<div className="flex flex-col items-center justify-center py-14 px-6 text-center">
					<motion.div
						animate={isDragging ? { scale: 1.15, rotate: -8 } : { scale: 1, rotate: 0 }}
						transition={{ type: "spring", stiffness: 300, damping: 20 }}
						className="mb-5 flex h-16 w-16 items-center justify-center rounded-2xl border border-white/10 bg-white/5 shadow-[inset_0_1px_0_rgba(255,255,255,0.1)]"
					>
						<Upload className="h-7 w-7 text-white/60 group-hover:text-white/90 transition-colors duration-200" strokeWidth={1.5} />
					</motion.div>

					<p className="text-[15px] font-medium text-white/80 mb-1">
						{isDragging ? "Drop your files here" : "Drag & drop or click to upload"}
					</p>
					<p className="text-xs text-white/35">
						Supports .eml files · Max {maxSize}MB each
					</p>
				</div>
			</div>

			{/* File list */}
			<AnimatePresence>
				{files.map((file, idx) => (
					<motion.div
						key={`${file.name}-${idx}`}
						initial={{ opacity: 0, y: -6, scale: 0.97 }}
						animate={{ opacity: 1, y: 0, scale: 1 }}
						exit={{ opacity: 0, x: 20, scale: 0.95 }}
						transition={{ duration: 0.2 }}
						className="flex items-center gap-3 rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3 backdrop-blur-sm"
					>
						<div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg border border-white/10 bg-white/5">
							<File className="h-4 w-4 text-white/60" strokeWidth={1.5} />
						</div>
						<div className="flex-1 min-w-0">
							<p className="text-sm font-medium text-white/90 truncate">{file.name}</p>
							<p className="text-xs text-white/35 mt-0.5">{formatSize(file.size)}</p>
						</div>
						<CheckCircle className="h-4 w-4 text-emerald-400 shrink-0" strokeWidth={1.5} />
						<button
							onClick={(e) => { e.stopPropagation(); removeFile(idx); }}
							className="ml-1 flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-white/40 hover:text-white/80 hover:bg-white/10 transition-all duration-150"
						>
							<X className="h-3.5 w-3.5" />
						</button>
					</motion.div>
				))}
			</AnimatePresence>

			{/* Error */}
			<AnimatePresence>
				{error && (
					<motion.p
						initial={{ opacity: 0, y: -4 }}
						animate={{ opacity: 1, y: 0 }}
						exit={{ opacity: 0 }}
						className="text-xs text-red-400 px-1"
					>
						{error}
					</motion.p>
				)}
			</AnimatePresence>
		</div>
	);
}
