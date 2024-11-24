from django.db.models.signals import pre_save

from storymint.models import Metadata


@pre_save(sender=Metadata)
def pre_save_metatdata(sender: type[Metadata], instance: Metadata, **kwargs) -> None:
    """Pre save metadata."""
    if not instance.image.name:
        instance.update_image()
    instance.update_metadata()
