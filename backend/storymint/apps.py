from django.apps import AppConfig


class StorymintConfig(AppConfig):
    """Storymint config."""

    default_auto_field = "django.db.models.BigAutoField"
    name = "storymint"

    def ready(self) -> None:
        """Ready."""
        from storymint import signals  # noqa F401
