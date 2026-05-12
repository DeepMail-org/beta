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
async def test_health_returns_ok(client):
    resp = await client.get("/health")
    assert resp.status_code == 200
    data = resp.json()
    assert data["status"] == "ok"
    assert "models" in data
    assert "redis" in data
    assert data["version"] == "0.1.0"


@pytest.mark.anyio
async def test_models_returns_list(client):
    resp = await client.get("/models")
    assert resp.status_code == 200
    data = resp.json()
    assert isinstance(data, list)
    for item in data:
        assert "name" in item
        assert "type" in item
        assert "loaded" in item
