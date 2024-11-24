import re

from django.db import models


def get_storage_url(field: models.Field, prefix: str) -> str:
    """Get storage URL without cruft."""
    url = field.storage.url(f"{prefix}/{field.name}")
    match = re.match(r"(.*)\?.*", url)
    if match:
        return match.group(1)
