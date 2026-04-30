import { cn } from "@/lib/utils";

export const BentoGrid = ({
	className,
	children,
}: {
	className?: string;
	children?: React.ReactNode;
}) => {
	return (
		<div
			className={cn(
				"mx-auto grid max-w-7xl grid-cols-1 gap-4 md:auto-rows-[18rem] md:grid-cols-3",
				className,
			)}
		>
			{children}
		</div>
	);
};

export const BentoGridItem = ({
	className,
	title,
	description,
	header,
	icon,
}: {
	className?: string;
	title?: string | React.ReactNode;
	description?: string | React.ReactNode;
	header?: React.ReactNode;
	icon?: React.ReactNode;
}) => {
	return (
		<div
			className={cn(
				"group/bento row-span-1 flex flex-col justify-between gap-4 rounded-2xl border border-white/8 bg-white/2 p-4 transition duration-200 hover:border-white/20 hover:bg-white/4 hover:shadow-[0_0_40px_rgba(255,255,255,0.06)]",
				className,
			)}
		>
			{header}
			<div className="transition duration-200 group-hover/bento:translate-x-2">
				{icon}
				<div className="mt-2 mb-2 font-display font-bold text-white/85">
					{title}
				</div>
				<div className="font-sans text-xs font-normal text-white/45 leading-relaxed">
					{description}
				</div>
			</div>
		</div>
	);
};
