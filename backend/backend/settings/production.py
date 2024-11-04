# ruff: noqa: F403, F405
import sys

import sentry_sdk
from decouple import config
from sentry_sdk.integrations.django import DjangoIntegration

from .base import *

# SECURITY WARNING: don't run with debug turned on in production!
DEBUG = False

ALLOWED_HOSTS = ["sol-warrior.com"]

USE_X_FORWARDED_HOST = True
SECURE_PROXY_SSL_HEADER = ("HTTP_X_FORWARDED_PROTO", "https")

sentry_sdk.init(
    dsn=config("SENTRY_DSN"),
    integrations=[DjangoIntegration()],
    # Less transactions
    traces_sample_rate=0.01,
)

STATICFILES_STORAGE = "whitenoise.storage.CompressedStaticFilesStorage"

# Database
# https://docs.djangoproject.com/en/5.1/ref/settings/#databases

DATABASES = {
    "default": {
        "ENGINE": "django.db.backends.postgresql_psycopg2",
        "NAME": config("DATABASE_NAME"),
        "USER": config("DATABASE_USER"),
        "PASSWORD": config("DATABASE_PASSWORD"),
        "HOST": config("DATABASE_HOST"),
        "PORT": config("PROXY_DATABASE_PORT", None),
        "TEST": {"NAME": f'test_{config("DATABASE_NAME")}'},
    },
}

LOGGING = {
    "version": 1,
    "disable_existing_loggers": False,
    "handlers": {
        "console": {
            "class": "logging.StreamHandler",
        },
    },
    "root": {
        "handlers": ["console"],
        "level": "INFO",
    },
}

# Media
DEFAULT_FILE_STORAGE = "storages.backends.gcloud.GoogleCloudStorage"
GS_BUCKET_NAME = (
    f'test-{config("GCS_BUCKET_NAME")}'
    if "test" in sys.argv
    else config("GCS_BUCKET_NAME")
)
