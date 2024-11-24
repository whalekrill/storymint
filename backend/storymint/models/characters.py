from django.db import models
from django.utils.module_loading import import_string
from django.utils.translation import gettext_lazy as _

from .base import AttributeBase, ImageBase, NameBase


class Character(NameBase, ImageBase, AttributeBase):
    """Character."""

    world = models.ForeignKey(
        "storymint.World", related_name="characters", on_delete=models.CASCADE
    )
    user = models.ForeignKey(
        "storymint.CustomUser", related_name="characters", on_delete=models.CASCADE
    )
    asset = models.CharField(
        _("asset"),
        help_text=_("Base58 encoded Solana public key"),
        max_length=44,
        unique=True,
        db_index=True,
    )
    current_node = models.ForeignKey("storymint.Node", on_delete=models.CASCADE)
    has_pending_metadata_update = models.BooleanField(
        _("has pending metadata update?"), default=False
    )

    def get_image_generator(self) -> str:
        """Get image generator."""
        return import_string(self.world.metadata.generator)

    class Meta:
        db_table = "storymint_character"
        verbose_name = _("character")
        verbose_name_plural = _("characters")


class CharacterChoice(models.Model):
    """Character choice."""

    character = models.ForeignKey(
        "storymint.Character",
        related_name="character_choices",
        on_delete=models.CASCADE,
    )
    choice = models.ForeignKey(
        "storymint.Choice", related_name="character_choices", on_delete=models.CASCADE
    )

    class Meta:
        db_table = "storymint_character_choice"
        verbose_name = _("character choice")
        verbose_name_plural = _("character choices")
