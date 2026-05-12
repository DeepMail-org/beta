from __future__ import annotations

import logging

from app.models.loader import get_model

logger = logging.getLogger(__name__)

_DANGEROUS_EXTENSIONS = {
    ".exe", ".scr", ".bat", ".cmd", ".com", ".pif", ".vbs", ".js",
    ".wsf", ".msi", ".dll", ".ps1", ".hta", ".cpl", ".reg",
}
_SUSPICIOUS_EXTENSIONS = {".zip", ".rar", ".7z", ".iso", ".img", ".docm", ".xlsm", ".pptm"}


def score_attachment(filename: str, size: int, content_type: str) -> dict:
    entry = get_model("attachment_scorer")
    if entry is not None:
        return _ml_score(filename, size, content_type, entry)
    return _rule_based_score(filename, size, content_type)


def _ml_score(filename: str, size: int, content_type: str, entry: dict) -> dict:
    try:
        from app.features.text_features import extract_attachment_features
        import numpy as np

        features = extract_attachment_features(filename, size, content_type)
        model = entry["model"]
        X = np.array([list(features.values())])
        proba = model.predict_proba(X)
        score = float(proba[0][1]) if proba.shape[1] > 1 else float(proba[0][0])
        risk_level = _score_to_risk(score)
        return {"score": round(score, 4), "risk_level": risk_level, "available": True}
    except Exception as e:
        logger.warning("ML attachment scoring failed: %s, falling back to rules", e)
        result = _rule_based_score(filename, size, content_type)
        result["available"] = True
        return result


def _rule_based_score(filename: str, size: int, content_type: str) -> dict:
    score = 0.0
    lower = filename.lower()

    for ext in _DANGEROUS_EXTENSIONS:
        if lower.endswith(ext):
            score += 0.6
            break

    if score == 0.0:
        for ext in _SUSPICIOUS_EXTENSIONS:
            if lower.endswith(ext):
                score += 0.3
                break

    parts = lower.rsplit(".", maxsplit=2)
    if len(parts) >= 3:
        score += 0.2

    if content_type == "application/octet-stream":
        score += 0.1

    if size > 25_000_000:
        score += 0.1

    score = min(score, 1.0)
    risk_level = _score_to_risk(score)

    return {"score": round(score, 4), "risk_level": risk_level, "available": False}


def _score_to_risk(score: float) -> str:
    if score >= 0.7:
        return "high"
    if score >= 0.4:
        return "medium"
    return "low"
