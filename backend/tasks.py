import os
import re
from pathlib import Path
from typing import Any

from decouple import config
from invoke import task


@task
def django_settings(ctx: Any) -> Any:
    """Get django settings."""
    os.environ["DJANGO_SETTINGS_MODULE"] = "backend.settings.development"
    import django

    django.setup()
    from django.conf import settings

    return settings


@task
def start_proxy(ctx: Any) -> None:
    """Start proxy."""
    host = config("PRODUCTION_DATABASE_HOST")
    port = config("PROXY_DATABASE_PORT")
    ctx.run(f'cloud-tools/cloud-sql-proxy -instances="{host}"=tcp:{port}')


@task
def build(ctx: Any) -> None:
    """Build the project."""
    delete_database(ctx)
    delete_media(ctx)
    delete_migrations(ctx)
    create_database(ctx)
    create_user(ctx)


@task
def create_database(ctx: Any) -> None:
    """Create the database."""
    ctx.run("python manage.py makemigrations")
    ctx.run("python manage.py migrate")


@task
def create_user(ctx: Any) -> None:
    """Create a superuser."""
    from django.contrib.auth import get_user_model

    User = get_user_model()
    user = User.objects.create(username="storymint", is_superuser=True, is_staff=True)
    user.set_password("storymint")
    user.save()


@task
def delete_database(ctx: any) -> None:
    """Delete the database."""
    django_settings(ctx)

    from django.conf import settings

    db = settings.BASE_DIR / "db.sqlite3"
    if db.exists():
        ctx.run(f"rm {db}")


@task
def delete_media(ctx: Any) -> None:
    """Delete media."""
    django_settings(ctx)

    from django.conf import settings

    if settings.MEDIA_ROOT.exists():
        ctx.run(f"rm -r {settings.MEDIA_ROOT}")


@task
def delete_migrations(ctx: Any) -> None:
    """Delete migrations."""
    import os

    from django.conf import settings

    MIGRATIONS_DIR = settings.BASE_DIR / "storymint/migrations/"

    migrations = [
        MIGRATIONS_DIR / migration
        for migration in os.listdir(MIGRATIONS_DIR)
        if Path(migration).stem != "__init__" and Path(migration).suffix == ".py"
    ]

    for migration in migrations:
        ctx.run(f"rm {migration}")


@task
def get_container_name(ctx: Any, region: str = "asia-northeast1") -> str:
    """Get container name."""
    project_id = ctx.run("gcloud config get-value project").stdout.strip()
    name = "storymint"
    return f"{region}-docker.pkg.dev/{project_id}/{name}/{name}"


def docker_secrets() -> str:
    """Get docker secrets."""
    build_args = [
        f'{secret}="{config(secret)}"' for secret in ("SECRET_KEY", "SENTRY_DSN")
    ]
    return " ".join([f"--build-arg {build_arg}" for build_arg in build_args])


def build_storymint(ctx: Any) -> str:
    """Build storymint."""
    result = ctx.run("poetry build").stdout
    return re.search(r"storymint-.*\.whl", result).group()


@task
def build_container(ctx: Any, region: str = "asia-northeast1") -> None:
    """Build container."""
    wheel = build_storymint(ctx)
    ctx.run("echo yes | python manage.py collectstatic")
    name = get_container_name(ctx, region=region)
    # Requirements
    requirements = [
        "django-filter",
        "django-taggit",
        "gunicorn",
        "pillow",
        "python-decouple",
        "whitenoise",
    ]
    # Versions
    reqs = " ".join(
        [
            req.split(";")[0]
            for req in ctx.run("poetry export --dev --without-hashes").stdout.split(
                "\n"
            )
            if req.split("==")[0] in requirements
        ]
    )
    # Build
    build_args = {"WHEEL": wheel, "POETRY_EXPORT": reqs}
    build_args = " ".join(
        [f'--build-arg {key}="{value}"' for key, value in build_args.items()]
    )
    with ctx.cd(".."):
        cmd = " ".join(
            [
                "docker build",
                build_args,
                docker_secrets(),
                f"--no-cache --file=Dockerfile --tag={name} .",
            ]
        )
        ctx.run(cmd)


@task
def push_container(ctx: Any, region: str = "asia-northeast1") -> None:
    """Push container."""
    name = get_container_name(ctx, region=region)
    # Push
    cmd = f"docker push {name}"
    ctx.run(cmd)
