FROM python:3.10-alpine

ARG WHEEL
ARG POETRY_EXPORT
ARG SECRET_KEY
ARG SENTRY_DSN

ENV SECRET_KEY $SECRET_KEY
ENV SENTRY_DSN $SENTRY_DSN

COPY dist/$WHEEL /
RUN mkdir storymint
COPY backend /storymint/backend
COPY storymint /storymint/storymint
COPY static /storymint/static
COPY media /storymint/media

RUN pip install --no-cache-dir wheel
RUN pip install $WHEEL
RUN pip install --no-cache-dir $POETRY_EXPORT sentry-sdk[django]
RUN rm $WHEEL

ENTRYPOINT ["gunicorn", "--chdir", "/storymint", "--bind", "0.0.0.0:8080", "--timeout", "0", "--preload", "demo.wsgi:application"]
