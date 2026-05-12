from __future__ import annotations

import hashlib
import time

from fastapi import APIRouter, Depends
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_session
from app.db.crud import log_inference
from app.models.url_model import classify_url
from app.redis_client import cache_get, cache_set

router = APIRouter()


class UrlRequest(BaseModel):
    url: str
    email_id: str | None = None


@router.post("/predict/url")
async def predict(req: UrlRequest, session: AsyncSession = Depends(get_session)):
    input_hash = hashlib.sha256(req.url.encode()).hexdigest()
    cache_key = f"url:{input_hash}"

    cached = await cache_get(cache_key)
    if cached:
        cached["cached"] = True
        return cached

    start = time.monotonic()
    result = classify_url(req.url)
    latency_ms = int((time.monotonic() - start) * 1000)

    result["cached"] = False
    await cache_set(cache_key, result)

    if session is not None:
        try:
            await log_inference(
                session,
                model_name="url_classifier",
                input_hash=input_hash,
                result_json=result,
                latency_ms=latency_ms,
                email_id=req.email_id,
            )
        except Exception:
            pass

    return result
