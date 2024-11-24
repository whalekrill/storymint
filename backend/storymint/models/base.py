import json
from io import BytesIO
from uuid import uuid4

from django.core.files.base import ContentFile
from django.db import models
from django.utils.translation import gettext_lazy as _

from storymint.serializers import MetadataSerializer


def get_default_attributes() -> dict:
    """Get default attributes."""
    return [
        {"trait_type": "Knowledge", "value": 1},
        {"trait_type": "Charm", "value": 1},
        {"trait_type": "Guts", "value": 1},
        {"trait_type": "Kindness", "value": 1},
        {"trait_type": "Proficiency", "value": 1},
    ]


class UUIDBase(models.Model):
    """UUID base."""

    uuid = models.UUIDField(_("uuid"), unique=True, db_index=True, default=uuid4)

    class Meta:
        abstract = True


class NameBase(models.Model):
    """Name base."""

    name = models.CharField(_("name"), max_length=255)

    def __str__(self) -> str:
        """str."""
        return self.name

    class Meta:
        abstract = True


class DescriptionBase(models.Model):
    """Description base."""

    description = models.TextField(_("description"))

    class Meta:
        abstract = True


class ImageBase(models.Model):
    """Image base."""

    image = models.ImageField(_("image"), upload_to="images", blank=True)

    def get_image_generator(self) -> str:
        """Get image generator."""
        raise NotImplementedError

    def update_image(self) -> None:
        """Update image."""
        ImageGenerator = self.get_image_generator()
        generator = ImageGenerator(self.attrs)
        buffer = generator.generate()
        self.image = ContentFile(buffer.getvalue())
        self.image.name = f"{self.uuid}.png"

    class Meta:
        abstract = True


class AttributeBase(models.Model):
    """Attribute base."""

    metadata = models.FileField(_("metadata"), upload_to="metadata", blank=True)
    attributes = models.JSONField(_("attributes"), default=get_default_attributes)
    max_value = models.PositiveIntegerField(_("max value"), default=5)

    @property
    def attrs(self) -> dict:
        """Attributes."""
        return {
            attr["trait_type"]: attr["value"] / self.max_value
            for attr in self.attributes
            if isinstance(attr["value"], int)
        }

    def update_metadata(self) -> None:
        """Update metadata."""
        buffer = BytesIO()
        serializer = MetadataSerializer(self)
        data = serializer.data
        data = json.dumps({k: v for k, v in data.items() if v})
        buffer.write(data.encode())
        self.metadata = ContentFile(buffer.getvalue())
        self.metadata.name = f"{self.uuid}.json"

    class Meta:
        abstract = True
