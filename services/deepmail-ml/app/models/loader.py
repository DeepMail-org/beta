from __future__ import annotations

import logging
from pathlib import Path
from typing import Any

from app.config import settings

logger = logging.getLogger(__name__)

_registry: dict[str, Any] = {}

MODEL_SPECS: list[dict[str, str]] = [
    {"name": "distilbert_phishing", "type": "transformers", "file": "distilbert_phishing"},
    {"name": "url_classifier", "type": "sklearn", "file": "url_classifier.joblib"},
    {"name": "attachment_scorer", "type": "xgboost", "file": "attachment_scorer.joblib"},
    {"name": "anomaly_detector", "type": "sklearn", "file": "anomaly_detector.joblib"},
    {"name": "email_embedder", "type": "sentence_transformers", "file": "email_embedder"},
]


async def load_all_models() -> dict[str, bool]:
    results: dict[str, bool] = {}
    model_dir = Path(settings.MODEL_DIR)

    for spec in MODEL_SPECS:
        name = spec["name"]
        model_path = model_dir / spec["file"]
        try:
            if spec["type"] == "transformers":
                results[name] = await _load_transformers(name, model_path)
            elif spec["type"] == "sentence_transformers":
                results[name] = await _load_sentence_transformers(name, model_path)
            elif spec["type"] in ("sklearn", "xgboost"):
                results[name] = _load_joblib(name, model_path)
            else:
                results[name] = False
        except Exception as e:
            logger.warning("failed to load %s: %s", name, e)
            results[name] = False

    loaded = sum(1 for v in results.values() if v)
    logger.info("loaded %d/%d models", loaded, len(results))
    return results


async def _load_transformers(name: str, path: Path) -> bool:
    if not path.exists():
        logger.info("model path %s not found, %s unavailable", path, name)
        return False
    try:
        from transformers import AutoModelForSequenceClassification, AutoTokenizer

        tokenizer = AutoTokenizer.from_pretrained(str(path))
        model = AutoModelForSequenceClassification.from_pretrained(str(path))
        model.eval()
        _registry[name] = {"model": model, "tokenizer": tokenizer}
        logger.info("loaded %s from %s", name, path)
        return True
    except Exception as e:
        logger.warning("failed to load transformers model %s: %s", name, e)
        return False


async def _load_sentence_transformers(name: str, path: Path) -> bool:
    if not path.exists():
        logger.info("model path %s not found, %s unavailable", path, name)
        return False
    try:
        from sentence_transformers import SentenceTransformer

        model = SentenceTransformer(str(path))
        _registry[name] = {"model": model}
        logger.info("loaded %s from %s", name, path)
        return True
    except Exception as e:
        logger.warning("failed to load sentence_transformers model %s: %s", name, e)
        return False


def _load_joblib(name: str, path: Path) -> bool:
    if not path.exists():
        logger.info("model path %s not found, %s unavailable", path, name)
        return False
    try:
        import joblib

        model = joblib.load(path)
        _registry[name] = {"model": model}
        logger.info("loaded %s from %s", name, path)
        return True
    except Exception as e:
        logger.warning("failed to load joblib model %s: %s", name, e)
        return False


def get_model(name: str) -> dict[str, Any] | None:
    return _registry.get(name)


def is_model_loaded(name: str) -> bool:
    return name in _registry
