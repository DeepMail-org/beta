from __future__ import annotations

import hashlib
import json
import time

from fastapi import APIRouter, Depends
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_session
from app.db.crud import log_inference
from app.models.anomaly import detect_anomaly
from app.redis_client import cache_get, cache_set

router = APIRouter()


class AnomalyRequest(BaseModel):
    features: dict
    email_id: str | None = None


@router.post("/predict/anomaly")
async def predict(req: AnomalyRequest, session: AsyncSession = Depends(get_session)):
    sorted_json = json.dumps(req.features, sort_keys=True)
    input_hash = hashlib.sha256(sorted_json.encode()).hexdigest()
    cache_key = f"anomaly:{input_hash}"

    cached = await cache_get(cache_key)
    if cached:
        cached["cached"] = True
        return cached

    start = time.monotonic()
    result = detect_anomaly(req.features)
    latency_ms = int((time.monotonic() - start) * 1000)

    result["cached"] = False
    await cache_set(cache_key, result)

    if session is not None:
        try:
            await log_inference(
                session,
                model_name="anomaly_detector",
                input_hash=input_hash,
                result_json=result,
                latency_ms=latency_ms,
                email_id=req.email_id,
            )
        except Exception:
            pass

    return result
