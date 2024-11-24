# ruff: noqa: F403, F405
import os
import sys

from decouple import config

from .base import *

# SECURITY WARNING: don't run with debug turned on in production!
DEBUG = True

# Database
# https://docs.djangoproject.com/en/4.0/ref/settings/#databases

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

# GCP
CREDENTIALS = BASE_DIR.parents[0] / "keys" / config("GOOGLE_APPLICATION_CREDENTIALS")
os.environ["GOOGLE_APPLICATION_CREDENTIALS"] = str(CREDENTIALS.resolve())

STORAGES = {
    "default": {
        "BACKEND": "storages.backends.gcloud.GoogleCloudStorage",
    }
}
GS_BUCKET_NAME = (
    f'test-{config("GCS_BUCKET_NAME")}'
    if "test" in sys.argv
    else config("GCS_BUCKET_NAME")
)
