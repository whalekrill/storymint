from django.db import models
from django.utils.module_loading import import_string
from django.utils.translation import gettext_lazy as _

from storymint.constants import ImageGenerator

from .base import AttributeBase, DescriptionBase, ImageBase, NameBase, UUIDBase


class Metadata(UUIDBase, NameBase, DescriptionBase, ImageBase, AttributeBase):
    """Asset Metadata."""

    world = models.OneToOneField(
        "storymint.World",
        related_name="metadata",
        on_delete=models.CASCADE,
        unique=True,
    )
    collection = models.CharField(
        _("collection"),
        help_text=_("Base58 encoded Solana public key"),
        max_length=44,
        unique=True,
        db_index=True,
        blank=True,
    )
    external_url = models.URLField(_("external url"), blank=True)
    generator = models.CharField(
        _("generator"),
        choices=ImageGenerator.choices,
        max_length=255,
        default=ImageGenerator.choices[0][0],
    )

    def get_image_generator(self) -> str:
        """Get image generator."""
        return import_string(f"storymint.{self.generator}")

    class Meta:
        db_table = "storymint_metadata"
        verbose_name = _("metadata")
        verbose_name_plural = _("metadata")
