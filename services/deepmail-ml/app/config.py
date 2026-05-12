from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    DATABASE_URL: str = "postgresql+asyncpg://deepmail:deepmailpw@localhost:5432/deepmail_ml"
    REDIS_URL: str = "redis://localhost:6379"
    QDRANT_URL: str = ""
    MODEL_DIR: str = "/models"
    ANOMALY_THRESHOLD: float = -0.1
    CACHE_TTL_SECS: int = 300
    MAX_TEXT_LENGTH: int = 10000
    LOG_LEVEL: str = "INFO"
    PORT: int = 8001
    WORKERS: int = 1


settings = Settings()
