import base58
from django.contrib.auth import get_user_model
from django.urls import reverse
from rest_framework import status
from rest_framework.exceptions import ErrorDetail
from rest_framework.test import APITestCase
from solathon import Keypair

User = get_user_model()


class SolanaAuthenticationTests(APITestCase):
    def setUp(self):
        self.keypair = Keypair()
        self.public_key = str(self.keypair.public_key)
        self.message = "Please sign this message."

    def create_valid_signature(
        self, message: str, keypair: Keypair | None = None
    ) -> str:
        """Create a valid signature using Ed25519"""
        key = keypair or self.keypair
        signed = key.sign(message)  # Assuming 'message' is bytes
        return base58.b58encode(signed.signature).decode("utf-8")

    def test_valid_signature(self):
        signature = self.create_valid_signature(self.message.encode())

        response = self.client.post(
            reverse("auth:signin_verify"),
            {
                "publicKey": self.public_key,
                "signature": signature,
                "signedMessage": self.message,
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_200_OK)
        self.assertIn("refresh", response.data)
        self.assertIn("access", response.data)
        self.assertTrue(User.objects.filter(username=self.public_key).exists())

    def test_invalid_signature(self):
        invalid_signature = self.create_valid_signature(b"Wrong message")

        response = self.client.post(
            reverse("auth:signin_verify"),
            {
                "publicKey": self.public_key,
                "signature": invalid_signature,
                "signedMessage": self.message,
            },
            format="json",
        )

        self.assertEqual(response.status_code, status.HTTP_400_BAD_REQUEST)
        self.assertEqual(
            response.data, [ErrorDetail(string="Invalid signature", code="invalid")]
        )
        self.assertFalse(User.objects.filter(username=self.public_key).exists())

    def test_signin_input_view(self):
        response = self.client.get(reverse("auth:signin_input"))
        self.assertEqual(response.status_code, status.HTTP_200_OK)
