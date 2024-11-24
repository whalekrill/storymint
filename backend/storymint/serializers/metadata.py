from django.db import models
from rest_framework import serializers

from storymint.utils import get_storage_url


class MetadataSerializer(serializers.Serializer):
    """Metadata serializer."""

    name = serializers.CharField()
    description = serializers.CharField()
    external_url = serializers.CharField()
    image = serializers.SerializerMethodField()
    attributes = serializers.JSONField()

    def get_image(self, obj: models.Model) -> str:
        """Get image."""
        return get_storage_url(obj.image)

    class Meta:
        fields = (
            "name",
            "description",
            "external_url",
            "image",
            "attributes",
        )
