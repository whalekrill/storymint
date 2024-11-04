from django.contrib.auth.models import AbstractUser
from django.db import models
from django.utils.translation import gettext_lazy as _


class CustomUser(AbstractUser):
    """Custom user."""

    username = models.CharField(
        _("username"),
        help_text=_("Base58 encoded Solana public key"),
        max_length=44,
        unique=True,
        db_index=True,
    )
    email = models.EmailField(_("email"), blank=True)

    class Meta:
        db_table = "storymint_user"
        verbose_name = _("user")
        verbose_name_plural = _("users")
