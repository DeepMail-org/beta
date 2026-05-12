from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from sqlalchemy import select, update
from sqlalchemy.ext.asyncio import AsyncSession

from app.db.inference_table import MlInferenceResult
from app.db.models_table import MlModelRecord
from app.db.training_table import MlTrainingJob


async def upsert_model_record(
    session: AsyncSession,
    model_name: str,
    model_type: str,
    version: str = "1.0.0",
    is_loaded: bool = False,
    model_path: str | None = None,
    accuracy: float | None = None,
) -> MlModelRecord:
    stmt = select(MlModelRecord).where(MlModelRecord.model_name == model_name)
    result = await session.execute(stmt)
    record = result.scalar_one_or_none()

    if record:
        await session.execute(
            update(MlModelRecord)
            .where(MlModelRecord.model_name == model_name)
            .values(
                model_type=model_type,
                version=version,
                is_loaded=is_loaded,
                model_path=model_path,
                accuracy=accuracy,
                loaded_at=datetime.now(timezone.utc) if is_loaded else record.loaded_at,
            )
        )
        await session.commit()
        result = await session.execute(stmt)
        record = result.scalar_one()
    else:
        record = MlModelRecord(
            model_name=model_name,
            model_type=model_type,
            version=version,
            is_loaded=is_loaded,
            model_path=model_path,
            accuracy=accuracy,
            loaded_at=datetime.now(timezone.utc) if is_loaded else None,
        )
        session.add(record)
        await session.commit()
        await session.refresh(record)

    return record


async def log_inference(
    session: AsyncSession,
    model_name: str,
    input_hash: str,
    result_json: dict[str, Any],
    latency_ms: int = 0,
    email_id: str | None = None,
) -> MlInferenceResult:
    import uuid as _uuid

    record = MlInferenceResult(
        model_name=model_name,
        input_hash=input_hash,
        result_json=result_json,
        latency_ms=latency_ms,
        email_id=_uuid.UUID(email_id) if email_id else None,
    )
    session.add(record)
    await session.commit()
    await session.refresh(record)
    return record


async def create_training_job(
    session: AsyncSession,
    model_name: str,
    samples_count: int = 0,
) -> MlTrainingJob:
    job = MlTrainingJob(
        model_name=model_name,
        samples_count=samples_count,
        status="pending",
    )
    session.add(job)
    await session.commit()
    await session.refresh(job)
    return job
