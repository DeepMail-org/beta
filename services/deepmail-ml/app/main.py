from __future__ import annotations

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.config import settings

logging.basicConfig(level=settings.LOG_LEVEL, format="%(asctime)s %(name)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    from app.database import close_db, create_tables
    from app.models.loader import load_all_models
    from app.redis_client import close_redis, connect_redis

    logger.info("starting deepmail-ml")
    try:
        await create_tables()
    except Exception as e:
        logger.warning("database unavailable (%s), persistence disabled", e)
    await connect_redis()
    status = await load_all_models()
    logger.info("model load status: %s", status)

    yield

    logger.info("shutting down deepmail-ml")
    await close_redis()
    await close_db()


app = FastAPI(title="deepmail-ml", version="0.1.0", lifespan=lifespan)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

from app.routers import anomaly, attachment, embeddings, health, phishing, url  # noqa: E402

app.include_router(health.router)
app.include_router(phishing.router)
app.include_router(url.router)
app.include_router(attachment.router)
app.include_router(anomaly.router)
app.include_router(embeddings.router)
