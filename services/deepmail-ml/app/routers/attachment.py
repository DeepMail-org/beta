from __future__ import annotations

import hashlib
import time

from fastapi import APIRouter, Depends
from pydantic import BaseModel
from sqlalchemy.ext.asyncio import AsyncSession

from app.database import get_session
from app.db.crud import log_inference
from app.models.attachment import score_attachment
from app.redis_client import cache_get, cache_set

router = APIRouter()


class AttachmentRequest(BaseModel):
    filename: str
    size: int
    content_type: str
    email_id: str | None = None


@router.post("/predict/attachment")
async def predict(req: AttachmentRequest, session: AsyncSession = Depends(get_session)):
    raw = f"{req.filename}|{req.size}|{req.content_type}"
    input_hash = hashlib.sha256(raw.encode()).hexdigest()
    cache_key = f"attachment:{input_hash}"

    cached = await cache_get(cache_key)
    if cached:
        cached["cached"] = True
        return cached

    start = time.monotonic()
    result = score_attachment(req.filename, req.size, req.content_type)
    latency_ms = int((time.monotonic() - start) * 1000)

    result["cached"] = False
    await cache_set(cache_key, result)

    if session is not None:
        try:
            await log_inference(
                session,
                model_name="attachment_scorer",
                input_hash=input_hash,
                result_json=result,
                latency_ms=latency_ms,
                email_id=req.email_id,
            )
        except Exception:
            pass

    return result
