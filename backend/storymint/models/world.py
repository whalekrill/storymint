from django.db import models
from django.utils.translation import gettext_lazy as _

from .base import DescriptionBase, NameBase


class World(NameBase, DescriptionBase):
    """World."""

    creator = models.ForeignKey(
        "storymint.CustomUser", related_name="worlds", on_delete=models.CASCADE
    )

    class Meta:
        db_table = "storymint_world"
        verbose_name = _("world")
        verbose_name_plural = _("worlds")


class Story(NameBase, DescriptionBase):
    """Story."""

    creator = models.ForeignKey(
        "storymint.CustomUser", related_name="stories", on_delete=models.CASCADE
    )
    world = models.ForeignKey(
        "storymint.World", related_name="stories", on_delete=models.CASCADE
    )
    prerequisites = models.ForeignKey(
        "self", related_name="storylines", on_delete=models.CASCADE, null=True
    )

    class Meta:
        db_table = "storymint_story"
        verbose_name = _("story")
        verbose_name_plural = _("story")


class StoryPath(NameBase, DescriptionBase):
    """Story path."""

    story = models.ForeignKey(
        "storymint.Story", related_name="paths", on_delete=models.CASCADE
    )

    class Meta:
        db_table = "storymint_story_path"
        verbose_name = _("story path")
        verbose_name_plural = _("story paths")


class Node(models.Model):
    """Node."""

    story_path = models.ForeignKey(
        "storymint.StoryPath", related_name="nodes", on_delete=models.CASCADE
    )
    text = models.TextField(_("text"))

    class Meta:
        db_table = "storymint_node"
        verbose_name = _("node")
        verbose_name_plural = _("nodes")


class Choice(models.Model):
    """Choice."""

    node = models.ForeignKey(
        "storymint.Node", related_name="choices", on_delete=models.CASCADE
    )
    next_node = models.ForeignKey(
        "storymint.Node",
        related_name="previous_nodes",
        on_delete=models.CASCADE,
        null=True,
    )
    choice = models.CharField(_("choice"), max_length=255)

    class Meta:
        db_table = "storymint_choice"
        verbose_name = _("choice")
        verbose_name_plural = _("choices")
