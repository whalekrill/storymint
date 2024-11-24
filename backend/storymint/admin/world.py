from django.db.models.query import QuerySet
from django.forms import Form
from django.http import HttpRequest
from semantic_admin import SemanticModelAdmin

from storymint.models import World


class StorymintWorldAdmin(SemanticModelAdmin):
    """Storymint world admin."""

    list_display = (
        "name",
        "description",
        "creator",
    )
    search_fields = ("name",)
    fields = ("name", "description")
    ordering = ("name",)

    def save_model(
        self, request: HttpRequest, obj: World, form: Form, change: bool
    ) -> None:
        """Save model."""
        if not change:
            obj.creator = request.user
        super().save_model(request, obj, form, change)

    def get_queryset(self, request: HttpRequest) -> QuerySet:
        """Get queryset."""
        return super().get_queryset(request).filter(creator=request.user)
