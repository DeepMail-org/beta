from __future__ import annotations

import logging

from app.config import settings
from app.models.loader import get_model

logger = logging.getLogger(__name__)


def predict_phishing(text: str) -> dict:
    entry = get_model("distilbert_phishing")
    if entry is None:
        return {"score": 0.0, "label": "unknown", "available": False}

    import torch

    model = entry["model"]
    tokenizer = entry["tokenizer"]

    truncated = text[: settings.MAX_TEXT_LENGTH]
    inputs = tokenizer(truncated, return_tensors="pt", truncation=True, max_length=512, padding=True)

    with torch.no_grad():
        outputs = model(**inputs)
        probs = torch.softmax(outputs.logits, dim=-1)

    phishing_score = probs[0][1].item() if probs.shape[1] > 1 else probs[0][0].item()
    label = "phishing" if phishing_score > 0.5 else "legitimate"

    return {"score": round(phishing_score, 4), "label": label, "available": True}
