from rest_framework.generics import RetrieveAPIView
from rest_framework.permissions import AllowAny

from storymint.models import Metadata
from storymint.serializers import MetadataSerializer


class StorymintMetadataView(RetrieveAPIView):
    """Storymint metadata."""

    permission_classes = (AllowAny,)
    serializer_class = MetadataSerializer
    queryset = Metadata.objects.all()
    lookup_field = "uuid"
