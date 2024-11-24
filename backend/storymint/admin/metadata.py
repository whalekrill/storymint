from django.db.models.query import QuerySet
from django.http import HttpRequest
from django.urls import reverse
from semantic_admin import SemanticModelAdmin

from storymint.models import Metadata


class StorymintMetadataAdmin(SemanticModelAdmin):
    """Storymint metadata admin."""

    list_display = (
        "world",
        "name",
        "description",
        "external_url",
        "image",
        "attributes",
    )
    search_fields = ("name",)
    fields = (
        ("name", "collection", "metadata_url"),
        "external_url",
        "description",
        "image",
        "attributes",
    )
    ordering = ("name",)

    def world(self, obj: Metadata) -> str:
        """World."""
        url = reverse("admin:storymint_world_change", args=[obj.world.pk])
        return f'<a href="{url}">{obj.world.name}</a>'

    def has_delete_permission(self, request: HttpRequest, obj: Metadata = None) -> bool:
        """Has delete permission."""
        return False

    def get_queryset(self, request: HttpRequest) -> QuerySet:
        """Get queryset."""
        return (
            super()
            .get_queryset(request)
            .select_related("world")
            .filter(world__creator=request.user)
        )
