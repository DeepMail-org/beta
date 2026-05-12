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
async def test_attachment_safe_file(client):
    with patch("app.routers.attachment.log_inference", new_callable=AsyncMock):
        resp = await client.post(
            "/predict/attachment",
            json={"filename": "report.pdf", "size": 1024, "content_type": "application/pdf"},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["risk_level"] == "low"
    assert data["available"] is False


@pytest.mark.anyio
async def test_attachment_dangerous_file(client):
    with patch("app.routers.attachment.log_inference", new_callable=AsyncMock):
        resp = await client.post(
            "/predict/attachment",
            json={"filename": "malware.exe", "size": 50000, "content_type": "application/octet-stream"},
        )
    assert resp.status_code == 200
    data = resp.json()
    assert data["score"] >= 0.6
    assert data["risk_level"] == "high"
