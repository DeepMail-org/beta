from __future__ import annotations

import logging
from typing import Any

from app.config import settings
from app.models.loader import get_model

logger = logging.getLogger(__name__)

_qdrant_client = None


def _get_qdrant():
    global _qdrant_client
    if _qdrant_client is not None:
        return _qdrant_client
    if not settings.QDRANT_URL:
        return None
    try:
        from qdrant_client import QdrantClient

        _qdrant_client = QdrantClient(url=settings.QDRANT_URL)
        return _qdrant_client
    except Exception as e:
        logger.warning("Qdrant unavailable: %s", e)
        return None


def generate_embedding(text: str) -> dict:
    entry = get_model("email_embedder")
    if entry is None:
        return {"embedding": [], "dimension": 0, "available": False}

    try:
        model = entry["model"]
        truncated = text[: settings.MAX_TEXT_LENGTH]
        embedding = model.encode(truncated).tolist()
        return {"embedding": embedding, "dimension": len(embedding), "available": True}
    except Exception as e:
        logger.warning("embedding generation failed: %s", e)
        return {"embedding": [], "dimension": 0, "available": True}


def store_embedding(email_id: str, text: str) -> bool:
    entry = get_model("email_embedder")
    if entry is None:
        return False

    client = _get_qdrant()
    if client is None:
        return False

    try:
        from qdrant_client.models import PointStruct

        model = entry["model"]
        truncated = text[: settings.MAX_TEXT_LENGTH]
        embedding = model.encode(truncated).tolist()

        client.upsert(
            collection_name="email_embeddings",
            points=[PointStruct(id=email_id, vector=embedding, payload={"email_id": email_id})],
        )
        return True
    except Exception as e:
        logger.warning("failed to store embedding: %s", e)
        return False


def search_similar(text: str, top_k: int = 5) -> list[dict[str, Any]]:
    entry = get_model("email_embedder")
    if entry is None:
        return []

    client = _get_qdrant()
    if client is None:
        return []

    try:
        model = entry["model"]
        truncated = text[: settings.MAX_TEXT_LENGTH]
        query_vec = model.encode(truncated).tolist()

        results = client.search(
            collection_name="email_embeddings",
            query_vector=query_vec,
            limit=top_k,
        )
        return [
            {"id": str(r.id), "score": round(r.score, 4), "payload": r.payload or {}}
            for r in results
        ]
    except Exception as e:
        logger.warning("similarity search failed: %s", e)
        return []
