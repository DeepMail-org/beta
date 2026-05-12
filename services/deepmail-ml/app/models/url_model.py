from __future__ import annotations

import logging
import re
from urllib.parse import urlparse

from app.models.loader import get_model

logger = logging.getLogger(__name__)

_SUSPICIOUS_TLDS = {".tk", ".ml", ".ga", ".cf", ".gq", ".xyz", ".top", ".buzz", ".club"}
_IP_PATTERN = re.compile(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}")


def classify_url(url: str) -> dict:
    entry = get_model("url_classifier")
    if entry is not None:
        return _ml_classify(url, entry)
    return _rule_based_classify(url)


def _ml_classify(url: str, entry: dict) -> dict:
    try:
        from app.features.url_features import extract_url_features

        features = extract_url_features(url)
        model = entry["model"]
        import numpy as np

        X = np.array([list(features.values())])
        proba = model.predict_proba(X)
        score = float(proba[0][1]) if proba.shape[1] > 1 else float(proba[0][0])
        category = "malicious" if score > 0.5 else "benign"
        return {"score": round(score, 4), "category": category, "available": True}
    except Exception as e:
        logger.warning("ML URL classification failed: %s, falling back to rules", e)
        result = _rule_based_classify(url)
        result["available"] = True
        return result


def _rule_based_classify(url: str) -> dict:
    score = 0.0
    parsed = urlparse(url)
    hostname = parsed.hostname or ""

    if _IP_PATTERN.search(hostname):
        score += 0.3

    if len(hostname) > 50:
        score += 0.2

    for tld in _SUSPICIOUS_TLDS:
        if hostname.endswith(tld):
            score += 0.2
            break

    if parsed.scheme == "http":
        score += 0.1

    if hostname.count(".") > 4:
        score += 0.1

    if "@" in url:
        score += 0.2

    score = min(score, 1.0)
    category = "suspicious" if score > 0.4 else "benign"

    return {"score": round(score, 4), "category": category, "available": False}
