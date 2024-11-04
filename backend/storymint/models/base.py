from django.db import models
from django.utils.translation import gettext_lazy as _


class NameBase(models.Model):
    """NameBase."""

    name = models.CharField(_("name"), max_length=255)

    def __str__(self) -> str:
        """str."""
        return self.name

    class Meta:
        abstract = True


class DescriptionBase(models.Model):
    """DescriptionBase."""

    description = models.TextField(_("description"))

    class Meta:
        abstract = True
