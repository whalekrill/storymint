import base58
from django.contrib.auth import get_user_model
from django.contrib.auth.backends import BaseBackend
from django.http import HttpRequest
from solathon.utils import verify_signature

User = get_user_model()


class SolanaAuthenticationBackend(BaseBackend):
    """Authentication backend for Solana wallet-based authentication."""

    def authenticate(
        self, request: HttpRequest, public_key: str, signature: str, signed_message: str
    ) -> User | None:
        """Authenticate."""
        try:
            verify_signature(
                base58.b58decode(public_key),
                base58.b58decode(signature),
                signed_message.encode("utf-8"),
            )
        except Exception:
            pass
        else:
            try:
                user = User.objects.get(address=public_key)
            except User.DoesNotExist:
                user = User.objects.create_user(username=public_key, address=public_key)
            return user
