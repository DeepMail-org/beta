import type { Metadata } from "next";
import "../globals.css";

export const metadata: Metadata = {
	title: "DeepMail — Email Threat Intelligence Platform",
	description:
		"Upload any .eml or .msg. Get IOC extraction, geo-intel enrichment, hop-timeline analysis, and threat scores — in seconds.",
};

export default function RootLayout({
	children,
}: Readonly<{
	children: React.ReactNode;
}>) {
	return (
		<html lang="en" className="dark" suppressHydrationWarning>
			<body className="antialiased font-body bg-(--background) text-(--foreground)">
				{children}
			</body>
		</html>
	);
}
