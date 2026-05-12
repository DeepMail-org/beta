from fastapi import APIRouter

from app.models.loader import MODEL_SPECS, is_model_loaded
from app.redis_client import is_connected as redis_connected

router = APIRouter()


@router.get("/health")
async def health():
    models_status = {spec["name"]: is_model_loaded(spec["name"]) for spec in MODEL_SPECS}
    return {
        "status": "ok",
        "models": models_status,
        "redis": redis_connected(),
        "version": "0.1.0",
    }


@router.get("/models")
async def list_models():
    return [
        {
            "name": spec["name"],
            "type": spec["type"],
            "loaded": is_model_loaded(spec["name"]),
        }
        for spec in MODEL_SPECS
    ]
