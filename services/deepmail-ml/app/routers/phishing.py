from __future__ import annotations

import hashlib
import time

from fastapi import APIRouter, Depends
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_session
from app.db.crud import log_inference
from app.models.distilbert import predict_phishing
from app.redis_client import cache_get, cache_set

router = APIRouter()


class PhishingRequest(BaseModel):
    text: str
    email_id: str | None = None


@router.post("/predict/phishing")
async def predict(req: PhishingRequest, session: AsyncSession = Depends(get_session)):
    input_hash = hashlib.sha256(req.text.encode()).hexdigest()
    cache_key = f"phishing:{input_hash}"

    cached = await cache_get(cache_key)
    if cached:
        cached["cached"] = True
        return cached

    start = time.monotonic()
    result = predict_phishing(req.text)
    latency_ms = int((time.monotonic() - start) * 1000)

    result["cached"] = False
    await cache_set(cache_key, result)

    if session is not None:
        try:
            await log_inference(
                session,
                model_name="distilbert_phishing",
                input_hash=input_hash,
                result_json=result,
                latency_ms=latency_ms,
                email_id=req.email_id,
            )
        except Exception:
            pass

    return result
