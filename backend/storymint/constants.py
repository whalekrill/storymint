from django.db import models
from django.utils.translation import gettext_lazy as _


class ImageGenerator(models.TextChoices):
    """Image generator."""

    STAR_CHART = "generators.StarChart", _("Star chart")
