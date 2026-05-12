from __future__ import annotations

import re
from urllib.parse import urlparse

_IP_PATTERN = re.compile(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")


def extract_url_features(url: str) -> dict:
    parsed = urlparse(url)
    hostname = parsed.hostname or ""
    path = parsed.path or ""
    query = parsed.query or ""

    return {
        "url_length": len(url),
        "hostname_length": len(hostname),
        "path_length": len(path),
        "num_dots": hostname.count("."),
        "num_hyphens": hostname.count("-"),
        "num_digits": sum(c.isdigit() for c in url),
        "has_ip": int(bool(_IP_PATTERN.search(hostname))),
        "has_at": int("@" in url),
        "is_https": int(parsed.scheme == "https"),
        "num_params": query.count("&") + (1 if query else 0),
        "num_subdomains": max(0, hostname.count(".") - 1),
    }
