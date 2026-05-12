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
async def test_url_benign(client):
    with patch("app.routers.url.log_inference", new_callable=AsyncMock):
        resp = await client.post("/predict/url", json={"url": "https://google.com"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["category"] == "benign"
    assert data["available"] is False


@pytest.mark.anyio
async def test_url_suspicious(client):
    with patch("app.routers.url.log_inference", new_callable=AsyncMock):
        resp = await client.post("/predict/url", json={"url": "http://192.168.1.1/login@evil.tk"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["score"] > 0.4
    assert data["category"] == "suspicious"
