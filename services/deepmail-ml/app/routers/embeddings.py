from __future__ import annotations

from fastapi import APIRouter
from pydantic import BaseModel

from app.models.embedder import generate_embedding, search_similar, store_embedding

router = APIRouter()


class EmbeddingRequest(BaseModel):
    text: str
    email_id: str | None = None


class SearchRequest(BaseModel):
    text: str
    top_k: int = 5


@router.post("/embeddings/generate")
async def generate(req: EmbeddingRequest):
    result = generate_embedding(req.text)
    if req.email_id:
        store_embedding(req.email_id, req.text)
    return result


@router.post("/embeddings/search")
async def search(req: SearchRequest):
    results = search_similar(req.text, req.top_k)
    return {"results": results}
