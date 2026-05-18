import type { NextConfig } from "next";

const nextConfig: NextConfig = {
	reactStrictMode: true,
	compress: true,
	productionBrowserSourceMaps: false,
	poweredByHeader: false,

	transpilePackages: [
		"@cloudscape-design/board-components",
		"@cloudscape-design/components",
		"@cloudscape-design/global-styles",
		"@cloudscape-design/component-toolkit",
	],

	experimental: {
		optimizePackageImports: [
			"lucide-react",
			"@tabler/icons-react",
			"@carbon/icons-react",
			"motion",
			"framer-motion",
			"date-fns",
		],
	},

	images: {
		formats: ["image/avif", "image/webp"],
		remotePatterns: [
			{ protocol: "https", hostname: "framerusercontent.com" },
			{ protocol: "https", hostname: "assets.aceternity.com" },
			{ protocol: "https", hostname: "images.unsplash.com" },
		],
	},
};

export default nextConfig;
