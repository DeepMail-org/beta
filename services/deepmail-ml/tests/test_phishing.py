from unittest.mock import AsyncMock, patch

import pytest
from httpx import ASGITransport, AsyncClient


@pytest.fixture
def anyio_backend():
    return "asyncio"


@pytest.fixture
async def client():
    with patch("app.database.create_tables", new_callable=AsyncMock), \
         patch("app.redis_client.connect_redis", new_callable=AsyncMock), \
         patch("app.models.loader.load_all_models", new_callable=AsyncMock, return_value={}):
        from app.main import app
        transport = ASGITransport(app=app)
        async with AsyncClient(transport=transport, base_url="http://test") as ac:
            yield ac


@pytest.mark.anyio
async def test_phishing_no_model(client):
    with patch("app.routers.phishing.log_inference", new_callable=AsyncMock):
        resp = await client.post("/predict/phishing", json={"text": "hello world"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["available"] is False
    assert data["score"] == 0.0
    assert data["label"] == "unknown"


@pytest.mark.anyio
async def test_phishing_with_email_id(client):
    with patch("app.routers.phishing.log_inference", new_callable=AsyncMock):
        resp = await client.post(
            "/predict/phishing",
            json={"text": "urgent action required", "email_id": "550e8400-e29b-41d4-a716-446655440000"},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert "score" in data
    assert "label" in data
