"use client";

import React, { useState } from "react";
import { cn } from "@/lib/utils";
import { Check, Copy, LucideIcon, Mail, MapPin, Phone, Clock, ChevronLeft, ChevronRight } from "lucide-react";
import { Button, ButtonProps } from "@/components/ui/button";
import { motion, AnimatePresence } from "framer-motion";
import {
	format,
	addMonths,
	subMonths,
	startOfMonth,
	endOfMonth,
	startOfWeek,
	endOfWeek,
	eachDayOfInterval,
	isSameMonth,
	isSameDay,
	isAfter,
	isBefore,
} from "date-fns";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";

const APP_EMAIL = "mail@example.com";
const APP_PHONE = "+91 1234567890";
const APP_PHONE_2 = "+91 0987654321";

export function ContactPage() {
	return (
		<div className="w-full">
			<div className="mx-auto max-w-6xl">
				<div className="flex grow flex-col justify-center px-4 md:px-6 pt-16 pb-8">
					<h1 className="text-4xl font-bold md:text-5xl">Contact Us</h1>
				</div>

				{/* Contact info boxes */}
				<div className="grid md:grid-cols-3">
					<Box icon={Mail} title="Email">
						<a
							href={`mailto:${APP_EMAIL}`}
							className="font-mono text-base font-medium tracking-wide hover:underline"
						>
							{APP_EMAIL}
						</a>
						<CopyButton className="size-6" test={APP_EMAIL} />
					</Box>
					<Box icon={MapPin} title="Office">
						<span className="font-mono text-base font-medium tracking-wide">
							NOC-1 Fourth Floor, BSDC, Jaipur, India
						</span>
					</Box>
					<Box
						icon={Phone}
						title="Phone"
						className="border-b-0 md:border-r-0"
					>
						<div>
							<div className="flex items-center gap-x-2">
								<a
									href={`tel:${APP_PHONE}`}
									className="block font-mono text-base font-medium tracking-wide hover:underline"
								>
									{APP_PHONE}
								</a>
								<CopyButton className="size-6" test={APP_PHONE} />
							</div>
							<div className="flex items-center gap-x-2">
								<a
									href={`tel:${APP_PHONE_2}`}
									className="block font-mono text-base font-medium tracking-wide hover:underline"
								>
									{APP_PHONE_2}
								</a>
								<CopyButton className="size-6" test={APP_PHONE_2} />
							</div>
						</div>
					</Box>
				</div>

				{/* Meeting Scheduler */}
				<div className="px-4 md:px-6 py-16">
					<div className="flex items-center gap-3 mb-8">
						<div className="flex h-10 w-10 items-center justify-center rounded-xl border border-white/10 bg-white/5">
							<Clock className="h-5 w-5 text-white/60" strokeWidth={1.5} />
						</div>
						<div>
							<h2 className="font-display text-2xl font-semibold">Schedule a Meeting</h2>
							<p className="text-sm text-muted-foreground mt-0.5">Pick a date range and we'll get back to you</p>
						</div>
					</div>
					<InlineScheduler />
				</div>
			</div>
		</div>
	);
}

function InlineScheduler() {
	const today = new Date();
	const [currentMonth, setCurrentMonth] = useState(startOfMonth(today));
	const [startDate, setStartDate] = useState<Date | null>(null);
	const [endDate, setEndDate] = useState<Date | null>(null);
	const [aiNotes, setAiNotes] = useState(false);
	const [scheduled, setScheduled] = useState(false);

	const weekDays = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

	const days = eachDayOfInterval({
		start: startOfWeek(startOfMonth(currentMonth), { weekStartsOn: 1 }),
		end: endOfWeek(endOfMonth(currentMonth), { weekStartsOn: 1 }),
	});

	const handleDateClick = (day: Date) => {
		if (!startDate || (startDate && endDate)) {
			setStartDate(day);
			setEndDate(null);
		} else if (isBefore(day, startDate)) {
			setStartDate(day);
		} else {
			setEndDate(day);
		}
	};

	const isSelected = (day: Date) =>
		(startDate && isSameDay(day, startDate)) ||
		(endDate && isSameDay(day, endDate)) ||
		false;

	const isInRange = (day: Date) =>
		!!(startDate && endDate && isAfter(day, startDate) && isBefore(day, endDate));

	const isStart = (day: Date) => !!(startDate && isSameDay(day, startDate));
	const isEnd = (day: Date) => !!(endDate && isSameDay(day, endDate));

	const handleSchedule = () => setScheduled(true);

	if (scheduled) {
		return (
			<motion.div
				initial={{ opacity: 0, scale: 0.97 }}
				animate={{ opacity: 1, scale: 1 }}
				className="flex flex-col items-center justify-center py-16 gap-4"
			>
				<div className="flex h-16 w-16 items-center justify-center rounded-full border border-white/15 bg-white/5">
					<Check className="h-7 w-7 text-emerald-400" strokeWidth={1.5} />
				</div>
				<h3 className="font-display text-xl font-semibold">Meeting Requested!</h3>
				<p className="text-sm text-muted-foreground text-center max-w-xs">
					{startDate && endDate
						? `${format(startDate, "MMM d")} – ${format(endDate, "MMM d, yyyy")}`
						: startDate ? format(startDate, "MMM d, yyyy") : ""}
					{" "}— We'll confirm shortly.
				</p>
				<button
					onClick={() => { setScheduled(false); setStartDate(null); setEndDate(null); }}
					className="mt-2 text-xs text-white/40 hover:text-white/70 transition-colors"
				>
					Schedule another
				</button>
			</motion.div>
		);
	}

	return (
		<div className="grid grid-cols-1 lg:grid-cols-2 gap-8 rounded-2xl border border-white/[0.07] bg-white/[0.02] p-6 lg:p-8">
			{/* Calendar */}
			<div>
				{/* Month navigation */}
				<div className="flex items-center justify-between mb-5">
					<Button variant="ghost" size="icon" onClick={() => setCurrentMonth(subMonths(currentMonth, 1))} aria-label="Previous month">
						<ChevronLeft className="h-4 w-4" />
					</Button>
					<AnimatePresence mode="wait">
						<motion.span
							key={format(currentMonth, "MMMM yyyy")}
							initial={{ opacity: 0, y: -8 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: 8 }}
							transition={{ duration: 0.18 }}
							className="text-base font-medium"
						>
							{format(currentMonth, "MMMM yyyy")}
						</motion.span>
					</AnimatePresence>
					<Button variant="ghost" size="icon" onClick={() => setCurrentMonth(addMonths(currentMonth, 1))} aria-label="Next month">
						<ChevronRight className="h-4 w-4" />
					</Button>
				</div>

				{/* Day headers */}
				<div className="grid grid-cols-7 mb-1">
					{weekDays.map((d) => (
						<div key={d} className="py-1 text-center text-xs font-medium text-white/35">{d}</div>
					))}
				</div>

				{/* Day cells */}
				<div className="grid grid-cols-7 gap-y-1">
					{days.map((day) => {
						const selected = isSelected(day);
						const inRange = isInRange(day);
						const start = isStart(day);
						const end = isEnd(day);
						const otherMonth = !isSameMonth(day, currentMonth);
						const isToday = isSameDay(day, today);

						return (
							<button
								key={day.toISOString()}
								onClick={() => handleDateClick(day)}
								className={cn(
									"relative h-9 w-9 mx-auto flex items-center justify-center text-sm transition-all duration-150 select-none",
									otherMonth && "text-white/20",
									!otherMonth && !selected && !inRange && "text-white/70 hover:bg-white/10 rounded-full",
									isToday && !selected && "font-bold text-white",
									inRange && "bg-white/10 text-white rounded-none",
									start && "bg-white text-black rounded-l-full rounded-r-none font-semibold",
									end && "bg-white text-black rounded-r-full rounded-l-none font-semibold",
									selected && !start && !end && "bg-white text-black rounded-full font-semibold",
								)}
							>
								{format(day, "d")}
								{isToday && !selected && (
									<span className="absolute bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-white/60" />
								)}
							</button>
						);
					})}
				</div>
			</div>

			{/* Right panel */}
			<div className="flex flex-col justify-between gap-6">
				<div className="space-y-4">
					{/* Start date display */}
					<div>
						<p className="text-xs font-medium text-white/40 uppercase tracking-wider mb-1.5">Start Date</p>
						<div className="flex items-center gap-3 rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3">
							<span className="text-sm flex-1 text-white/80">
								{startDate ? format(startDate, "MMMM d, yyyy") : "Click a day to select"}
							</span>
						</div>
					</div>

					{/* End date display */}
					<div>
						<p className="text-xs font-medium text-white/40 uppercase tracking-wider mb-1.5">End Date</p>
						<div className="flex items-center gap-3 rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3">
							<span className="text-sm flex-1 text-white/80">
								{endDate ? format(endDate, "MMMM d, yyyy") : startDate ? "Click another day" : "—"}
							</span>
						</div>
					</div>

					{/* AI Notes toggle */}
					<div className="flex items-center justify-between rounded-xl border border-white/10 bg-white/[0.03] px-4 py-3">
						<div>
							<Label htmlFor="ai-notes-contact" className="text-sm font-medium text-white/80">AI Meeting Notes</Label>
							<p className="text-xs text-white/35 mt-0.5">Auto-generate agenda & summaries</p>
						</div>
						<Switch id="ai-notes-contact" checked={aiNotes} onCheckedChange={setAiNotes} />
					</div>
				</div>

				{/* Actions */}
				<div className="space-y-3 pt-2 border-t border-white/[0.06]">
					{startDate && (
						<p className="text-xs text-white/40">
							{endDate
								? `${format(startDate, "MMM d")} – ${format(endDate, "MMM d, yyyy")}`
								: `From ${format(startDate, "MMMM d, yyyy")}`}
						</p>
					)}
					<div className="flex gap-3">
						<button
							onClick={() => { setStartDate(null); setEndDate(null); }}
							className="flex-1 h-11 rounded-full text-sm font-medium border border-white/15 text-white/60 hover:text-white/90 hover:border-white/30 transition-all duration-200"
						>
							Clear
						</button>
						<button
							onClick={handleSchedule}
							disabled={!startDate}
							className="flex-1 h-11 rounded-full text-sm font-medium liquid-glass text-white border border-white/20 shadow-[0_0_18px_rgba(255,255,255,0.07)] hover:shadow-[0_0_24px_rgba(255,255,255,0.12)] transition-all duration-300 disabled:opacity-40 disabled:cursor-not-allowed"
						>
							Schedule Meeting
						</button>
					</div>
				</div>
			</div>
		</div>
	);
}

type ContactBox = React.ComponentProps<"div"> & {
	icon: LucideIcon;
	title: string;
};

function Box({ title, className, children, ...props }: ContactBox) {
	return (
		<div
			className={cn(
				"flex flex-col justify-between border-b md:border-r md:border-b-0",
				className,
			)}
		>
			<div className="bg-muted/40 flex items-center gap-x-3 border-b p-4">
				<props.icon className="text-muted-foreground size-5" strokeWidth={1} />
				<h2 className="font-heading text-lg font-medium tracking-wider">{title}</h2>
			</div>
			<div className="flex items-center gap-x-2 p-4 py-12">{children}</div>
			<div className="border-t p-0"></div>
		</div>
	);
}

type CopyButtonProps = ButtonProps & { test: string };

function CopyButton({ className, variant = "ghost", size = "icon", test, ...props }: CopyButtonProps) {
	const [copied, setCopied] = React.useState(false);

	const handleCopy = async () => {
		try {
			await navigator.clipboard.writeText(test);
			setCopied(true);
			setTimeout(() => setCopied(false), 1500);
		} catch (err) {
			console.error("Failed to copy text: ", err);
		}
	};

	return (
		<Button
			variant={variant}
			size={size}
			className={cn("disabled:opacity-100", className)}
			onClick={handleCopy}
			aria-label={copied ? "Copied" : "Copy to clipboard"}
			disabled={copied || props.disabled}
			{...props}
		>
			<div className={cn("transition-all", copied ? "scale-100 opacity-100" : "scale-0 opacity-0")}>
				<Check className="size-3.5 stroke-emerald-500" aria-hidden="true" />
			</div>
			<div className={cn("absolute transition-all", copied ? "scale-0 opacity-0" : "scale-100 opacity-100")}>
				<Copy aria-hidden="true" className="size-3.5" />
			</div>
		</Button>
	);
}
