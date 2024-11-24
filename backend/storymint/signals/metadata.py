from uuid import uuid4

from django.db.models.signals import pre_save
from django.dispatch import receiver

from storymint.models import Metadata


@receiver(pre_save, sender=Metadata, dispatch_uid=uuid4())
def pre_save_metadata(sender: type[Metadata], instance: Metadata, **kwargs) -> None:
    """Pre save metadata."""
    instance.image.delete(save=False)
    instance.metadata.delete(save=False)
    instance.update_image()
    instance.update_metadata()
