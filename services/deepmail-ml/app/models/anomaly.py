from __future__ import annotations

import logging

from app.config import settings
from app.models.loader import get_model

logger = logging.getLogger(__name__)


def detect_anomaly(features: dict) -> dict:
    entry = get_model("anomaly_detector")
    if entry is None:
        return {"score": 0.0, "is_anomaly": False, "available": False}

    try:
        import numpy as np

        model = entry["model"]
        X = np.array([list(features.values())])
        raw_score = float(model.decision_function(X)[0])
        is_anomaly = raw_score < settings.ANOMALY_THRESHOLD
        normalized = max(0.0, min(1.0, -raw_score))

        return {"score": round(normalized, 4), "is_anomaly": is_anomaly, "available": True}
    except Exception as e:
        logger.warning("anomaly detection failed: %s", e)
        return {"score": 0.0, "is_anomaly": False, "available": True}
