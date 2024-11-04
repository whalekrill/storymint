from django.contrib.auth import authenticate, login
from django.utils.timezone import localtime
from rest_framework import serializers, views
from rest_framework.permissions import AllowAny
from rest_framework.request import Request
from rest_framework.response import Response
from rest_framework_simplejwt.tokens import RefreshToken


class SolanaSignInInputView(views.APIView):
    """API endpoint to request a sign-in message."""

    permission_classes = (AllowAny,)

    def get(self, request: Request, *args, **kwargs) -> Response:
        """GET."""
        return Response(
            {
                "statement": "Please sign this message to verify your wallet ownership.",
                "issuedAt": localtime().isoformat(),
            }
        )


class SolanaSignInVerificationView(views.APIView):
    """API endpoint to verify a Solana sign-in message."""

    permission_classes = (AllowAny,)

    def post(self, request: Request, *args, **kwargs) -> Response:
        """POST."""
        data = request.data
        user = authenticate(
            request,
            public_key=data.get("publicKey"),
            signature=data.get("signature"),
            signed_message=data.get("signedMessage"),
        )
        if not user:
            raise serializers.ValidationError("Invalid signature")
        else:
            login(request, user)
            refresh = RefreshToken.for_user(user)
            return Response(
                {"refresh": str(refresh), "access": str(refresh.access_token)}
            )
