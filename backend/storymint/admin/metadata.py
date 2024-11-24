from django.contrib import admin
from django.db import models
from django.db.models.query import QuerySet
from django.http import HttpRequest
from django.urls import reverse
from django.utils.html import format_html
from django_svelte_jsoneditor.widgets import SvelteJSONEditorWidget
from semantic_admin import SemanticModelAdmin

from storymint.models import Metadata
from storymint.utils import get_storage_url


@admin.register(Metadata)
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
        (
            "world",
            "name",
        ),
        ("collection", "metadata_url"),
        "description",
        ("image", "external_url"),
        "attributes",
    )
    readonly_fields = ("metadata_url",)
    formfield_overrides = {
        models.JSONField: {
            "widget": SvelteJSONEditorWidget,
        }
    }
    ordering = ("name",)

    def world(self, obj: Metadata) -> str:
        """World."""
        url = reverse("admin:storymint_world_change", args=[obj.world.pk])
        return f'<a href="{url}">{obj.world.name}</a>'

    def metadata_url(self, obj: Metadata) -> str:
        """Metadata URL."""
        if obj.metadata:
            url = get_storage_url(obj.metadata)
            return format_html(f'<a href="{url}">{url}</a>')
        else:
            return "-"

    def get_readonly_fields(self, request: HttpRequest, obj: Metadata = None) -> tuple:
        """Get readonly fields."""
        if obj:
            return self.readonly_fields + ("world",)
        return self.readonly_fields

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
