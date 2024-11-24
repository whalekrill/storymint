import re

from django.db import models


def get_storage_url(field: models.Field) -> str:
    """Get storage URL without cruft."""
    match = re.match(r"(.*)\?.*", field.url)
    if match:
        return match.group(1)
