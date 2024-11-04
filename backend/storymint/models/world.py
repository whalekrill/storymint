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


class Faction(NameBase, DescriptionBase):
    """Faction."""

    world = models.ForeignKey(
        "storymint.World", related_name="factions", on_delete=models.CASCADE
    )

    class Meta:
        db_table = "storymint_faction"
        verbose_name = _("faction")
        verbose_name_plural = _("factions")


class CharacterAttribute(NameBase, DescriptionBase):
    """Attribute."""

    world = models.ForeignKey(
        "storymint.Story", related_name="character_attributes", on_delete=models.CASCADE
    )

    class Meta:
        db_table = "storymint_character_attribute"
        verbose_name = _("character attribute")
        verbose_name_plural = _("character attributes")


class Story(NameBase, DescriptionBase):
    """Story."""

    creator = models.ForeignKey(
        "storymint.CustomUser", related_name="stories", on_delete=models.CASCADE
    )
    world = models.ForeignKey(
        "storymint.World", related_name="stories", on_delete=models.CASCADE
    )
    prerequisite = models.ForeignKey(
        "self", related_name="next_stories", on_delete=models.CASCADE, null=True
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


class Character(NameBase):
    """Character."""

    faction = models.ForeignKey(
        "storymint.Faction", related_name="characters", on_delete=models.CASCADE
    )
    user = models.ForeignKey(
        "storymint.CustomUser", related_name="characters", on_delete=models.CASCADE
    )
    address = models.CharField(_("address"), max_length=44, db_index=True)
    current_node = models.ForeignKey("storymint.Node", on_delete=models.CASCADE)
    attrs = models.JSONField(_("attrs"), default=dict)

    class Meta:
        db_table = "storymint_character"
        verbose_name = _("character")
        verbose_name_plural = _("characters")


class Node(models.Model):
    """Node."""

    story = models.ForeignKey(
        "storymint.Story", related_name="nodes", on_delete=models.CASCADE
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


class CharacterChoice(models.Model):
    """Character choice."""

    story = models.ForeignKey(
        "storymint.Story", related_name="character_choices", on_delete=models.CASCADE
    )
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
