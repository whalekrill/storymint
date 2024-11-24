import json
from io import StringIO
from uuid import uuid4

from django.core.files.base import ContentFile
from django.db import models
from django.utils.translation import gettext_lazy as _

from storymint.serializers import MetadataSerializer


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
        if not self.image.name:
            ImageGenerator = self.get_image_generator()
            self.image = ImageGenerator(self.attrs)
            ImageGenerator = self.generator
            self.image = ImageGenerator(self.attrs)
            self.image.name = f"{self.uuid}.png"

    class Meta:
        abstract = True


class AttributeBase(models.Model):
    """Attribute base."""

    metadata = models.FileField(_("metadata"), upload_to="metadata", blank=True)
    attributes = models.JSONField(_("attributes"), default=list)

    @property
    def attrs(self) -> dict:
        """Attributes."""
        return {attr["trait_value"]: attr["value"] for attr in self.attributes}

    def update_metadata(self) -> None:
        """Update metadata."""
        buffer = StringIO()
        serializer = MetadataSerializer(self)
        data = json.dumps(serializer.data)
        buffer.write(data)
        self.metadata = ContentFile(buffer.getvalue())
        self.metadata.name = f"{self.uuid}.json"

    class Meta:
        abstract = True
