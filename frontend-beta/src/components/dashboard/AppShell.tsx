"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useMemo,
	useState,
	type ReactNode,
} from "react";

type IconName =
	| "grid"
	| "mail"
	| "cube"
	| "radar"
	| "folder"
	| "lines"
	| "report"
	| "cog"
	| "bell"
	| "search"
	| "menu"
	| "spark"
	| "logo";

function Glyph({ name, className = "" }: { name: IconName; className?: string }) {
	const common = {
		viewBox: "0 0 24 24",
		fill: "none",
		stroke: "currentColor",
		strokeWidth: 1.6,
		strokeLinecap: "round" as const,
		strokeLinejoin: "round" as const,
		className,
	};
	switch (name) {
		case "grid":
			return (
				<svg {...common}>
					<rect x="3" y="3" width="7" height="7" rx="1.5" />
					<rect x="14" y="3" width="7" height="7" rx="1.5" />
					<rect x="3" y="14" width="7" height="7" rx="1.5" />
					<rect x="14" y="14" width="7" height="7" rx="1.5" />
				</svg>
			);
		case "mail":
			return (
				<svg {...common}>
					<rect x="3" y="5" width="18" height="14" rx="2" />
					<path d="m3 7 9 6 9-6" />
				</svg>
			);
		case "cube":
			return (
				<svg {...common}>
					<path d="M12 2 3 7v10l9 5 9-5V7l-9-5Z" />
					<path d="m3 7 9 5 9-5" />
					<path d="M12 12v10" />
				</svg>
			);
		case "radar":
			return (
				<svg {...common}>
					<circle cx="12" cy="12" r="9" />
					<circle cx="12" cy="12" r="5" />
					<circle cx="12" cy="12" r="1.5" fill="currentColor" />
					<path d="M12 3v9l7 4" />
				</svg>
			);
		case "folder":
			return (
				<svg {...common}>
					<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z" />
				</svg>
			);
		case "lines":
			return (
				<svg {...common}>
					<path d="M4 6h16M4 12h10M4 18h16" />
				</svg>
			);
		case "report":
			return (
				<svg {...common}>
					<path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9l-6-6Z" />
					<path d="M14 3v6h6M8 13h8M8 17h5" />
				</svg>
			);
		case "cog":
			return (
				<svg {...common}>
					<circle cx="12" cy="12" r="3" />
					<path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1Z" />
				</svg>
			);
		case "bell":
			return (
				<svg {...common}>
					<path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
					<path d="M10 21a2 2 0 0 0 4 0" />
				</svg>
			);
		case "search":
			return (
				<svg {...common}>
					<circle cx="11" cy="11" r="7" />
					<path d="m20 20-3.5-3.5" />
				</svg>
			);
		case "menu":
			return (
				<svg {...common}>
					<path d="M4 6h16M4 12h16M4 18h16" />
				</svg>
			);
		case "spark":
			return (
				<svg {...common}>
					<path d="M12 2v4M12 18v4M2 12h4M18 12h4M5 5l3 3M16 16l3 3M5 19l3-3M16 8l3-3" />
				</svg>
			);
		case "logo":
			return (
				<svg {...common} strokeWidth={2}>
					<path d="M3 5h12a6 6 0 0 1 0 12H3z" />
					<path d="m3 5 9 7 9-7" />
				</svg>
			);
	}
}

type NavItem = { href: string; label: string; icon: IconName };

const NAV_PRIMARY: NavItem[] = [
	{ href: "/dashboard", label: "Dashboard", icon: "grid" },
	{ href: "/analysis", label: "Email Analysis", icon: "mail" },
	{ href: "/sandbox", label: "Sandbox", icon: "cube" },
	{ href: "/threat-intel", label: "Threat Intel", icon: "radar" },
	{ href: "/cases", label: "Cases", icon: "folder" },
];

const NAV_SECONDARY: NavItem[] = [
	{ href: "/logs", label: "Email Logs", icon: "lines" },
	{ href: "/reports", label: "Reports", icon: "report" },
	{ href: "/settings", label: "Settings", icon: "cog" },
];

type ShellMenuCtx = { openMobile: () => void; isMobileOpen: boolean };
const ShellMenuContext = createContext<ShellMenuCtx>({
	openMobile: () => {},
	isMobileOpen: false,
});
export const useShellMenu = () => useContext(ShellMenuContext);

function Sidebar({
	pathname,
	mobileOpen,
	onClose,
}: {
	pathname: string;
	mobileOpen: boolean;
	onClose: () => void;
}) {
	const isActive = (href: string) =>
		pathname === href || pathname.startsWith(href + "/");

	return (
		<aside
			className={`hf-sidebar ${mobileOpen ? "is-open" : ""}`}
			aria-label="Primary"
		>
			<div className="hf-sidebar-top">
				<div className="hf-logo">
					<Glyph name="logo" className="w-5 h-5" />
				</div>
				<div className="hf-logo-text">
					Deep<span>mail</span>
				</div>
				<button
					type="button"
					className="hf-hamburger md:hidden ml-auto"
					onClick={onClose}
					aria-label="Close menu"
				>
					<Glyph name="menu" className="w-5 h-5" />
				</button>
			</div>

			<nav className="hf-nav" aria-label="Primary navigation">
				{NAV_PRIMARY.map((item) => {
					const active = isActive(item.href);
					return (
						<Link
							key={item.href}
							href={item.href}
							onClick={onClose}
							className={`hf-nav-item ${active ? "is-active" : ""}`}
							aria-current={active ? "page" : undefined}
						>
							<span className="hf-nav-icon">
								<Glyph name={item.icon} className="w-4 h-4" />
							</span>
							<span className="hf-nav-label">{item.label}</span>
							{active && <span className="hf-nav-indicator" />}
						</Link>
					);
				})}
				<div className="hf-nav-divider" />
				{NAV_SECONDARY.map((item) => {
					const active = isActive(item.href);
					return (
						<Link
							key={item.href}
							href={item.href}
							onClick={onClose}
							className={`hf-nav-item ${active ? "is-active" : ""}`}
							aria-current={active ? "page" : undefined}
						>
							<span className="hf-nav-icon">
								<Glyph name={item.icon} className="w-4 h-4" />
							</span>
							<span className="hf-nav-label">{item.label}</span>
							{active && <span className="hf-nav-indicator" />}
						</Link>
					);
				})}
			</nav>

			<div className="hf-sidebar-bottom">
				<div className="flex items-center gap-3">
					<div className="hf-avatar">VJ</div>
					<div className="min-w-0">
						<div className="text-xs font-bold truncate">Vyom J.</div>
						<div className="text-[10px] opacity-60 flex items-center gap-1.5">
							<span className="hf-plan-dot" />
							Pro · Tenant 01
						</div>
					</div>
				</div>
			</div>
		</aside>
	);
}

function Topbar({
	title,
	breadcrumb,
	onOpenMenu,
}: {
	title: string;
	breadcrumb?: string;
	onOpenMenu: () => void;
}) {
	return (
		<header className="hf-topbar">
			<div className="hf-topbar-left">
				<button
					type="button"
					className="hf-hamburger md:hidden"
					onClick={onOpenMenu}
					aria-label="Open menu"
				>
					<Glyph name="menu" className="w-5 h-5" />
				</button>
				<div className="min-w-0">
					{breadcrumb && (
						<div className="hf-breadcrumb">{breadcrumb}</div>
					)}
					<h1 className="hf-page-title truncate">{title}</h1>
				</div>
			</div>
			<div className="hf-topbar-right">
				<button
					type="button"
					className="hf-topbar-icon"
					aria-label="Search"
				>
					<Glyph name="search" className="w-4 h-4" />
				</button>
				<button
					type="button"
					className="hf-topbar-icon relative"
					aria-label="Notifications"
				>
					<Glyph name="bell" className="w-4 h-4" />
					<span className="hf-notif-dot" aria-hidden />
				</button>
				<div className="hf-avatar-sm" aria-hidden>
					VJ
				</div>
			</div>
		</header>
	);
}

export default function AppShell({
	title,
	breadcrumb,
	children,
}: {
	title: string;
	breadcrumb?: string;
	children: ReactNode;
}) {
	const pathname = usePathname() || "/";
	const [mobileOpen, setMobileOpen] = useState(false);

	const close = useCallback(() => setMobileOpen(false), []);
	const open = useCallback(() => setMobileOpen(true), []);

	useEffect(() => {
		if (!mobileOpen) return;
		const onKey = (e: KeyboardEvent) => {
			if (e.key === "Escape") close();
		};
		document.addEventListener("keydown", onKey);
		return () => document.removeEventListener("keydown", onKey);
	}, [mobileOpen, close]);

	useEffect(() => {
		close();
	}, [pathname, close]);

	const ctx = useMemo(
		() => ({ openMobile: open, isMobileOpen: mobileOpen }),
		[open, mobileOpen],
	);

	return (
		<ShellMenuContext.Provider value={ctx}>
			<div className="hf-root">
				<div className="hf-shell">
					<Sidebar
						pathname={pathname}
						mobileOpen={mobileOpen}
						onClose={close}
					/>
					{mobileOpen && (
						<button
							type="button"
							className="hf-sidebar-scrim"
							onClick={close}
							aria-label="Close menu overlay"
						/>
					)}
					<div className="hf-main">
						<Topbar
							title={title}
							breadcrumb={breadcrumb}
							onOpenMenu={open}
						/>
						<main className="hf-content hf-fade-in">{children}</main>
					</div>
				</div>
			</div>
		</ShellMenuContext.Provider>
	);
}
