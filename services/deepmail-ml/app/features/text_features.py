from __future__ import annotations

_EXECUTABLE_TYPES = {
    "application/x-executable", "application/x-msdos-program",
    "application/x-msdownload", "application/vnd.microsoft.portable-executable",
}
_ARCHIVE_TYPES = {
    "application/zip", "application/x-rar-compressed", "application/x-7z-compressed",
    "application/gzip", "application/x-tar",
}
_DOCUMENT_TYPES = {
    "application/pdf", "application/msword",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
}


def extract_attachment_features(filename: str, size: int, content_type: str) -> dict:
    lower = filename.lower()
    parts = lower.rsplit(".", maxsplit=2)
    ext = f".{parts[-1]}" if len(parts) > 1 else ""

    return {
        "filename_length": len(filename),
        "extension_length": len(ext),
        "has_double_ext": int(len(parts) >= 3),
        "size_kb": size // 1024,
        "is_executable_type": int(content_type in _EXECUTABLE_TYPES),
        "is_archive_type": int(content_type in _ARCHIVE_TYPES),
        "is_document_type": int(content_type in _DOCUMENT_TYPES),
    }
