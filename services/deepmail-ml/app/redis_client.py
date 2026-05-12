from __future__ import annotations

import json
import logging
from typing import Any

import redis.asyncio as redis

from app.config import settings

logger = logging.getLogger(__name__)

_redis: redis.Redis | None = None


async def connect_redis() -> None:
    global _redis
    try:
        _redis = redis.from_url(settings.REDIS_URL, decode_responses=True)
        await _redis.ping()
        logger.info("connected to Redis")
    except Exception as e:
        logger.warning("Redis unavailable (%s), caching disabled", e)
        _redis = None


async def close_redis() -> None:
    global _redis
    if _redis:
        await _redis.aclose()
        _redis = None


async def cache_get(key: str) -> dict[str, Any] | None:
    if _redis is None:
        return None
    try:
        data = await _redis.get(key)
        if data:
            return json.loads(data)
    except Exception:
        pass
    return None


async def cache_set(key: str, value: dict[str, Any], ttl: int | None = None) -> None:
    if _redis is None:
        return
    try:
        await _redis.set(key, json.dumps(value), ex=ttl or settings.CACHE_TTL_SECS)
    except Exception:
        pass


def is_connected() -> bool:
    return _redis is not None
