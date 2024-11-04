"""
URL configuration for backend project.

The `urlpatterns` list routes URLs to views. For more information please see:
    https://docs.djangoproject.com/en/5.1/topics/http/urls/
Examples:
Function views
    1. Add an import:  from my_app import views
    2. Add a URL to urlpatterns:  path('', views.home, name='home')
Class-based views
    1. Add an import:  from other_app.views import Home
    2. Add a URL to urlpatterns:  path('', Home.as_view(), name='home')
Including another URLconf
    1. Import the include() function: from django.urls import include, path
    2. Add a URL to urlpatterns:  path('blog/', include('blog.urls'))
"""

from django.conf import settings
from django.contrib import admin
from django.urls import include, path
from rest_framework_simplejwt.views import TokenRefreshView

from storymint.views import SolanaSignInInputView, SolanaSignInVerificationView

auth_patterns = [
    path("signin/", SolanaSignInInputView.as_view(), name="signin_input"),
    path("verify/", SolanaSignInVerificationView.as_view(), name="signin_verify"),
    path("refresh/", TokenRefreshView.as_view(), name="token_refresh"),
]

urlpatterns = [
    path("api/auth/", include((auth_patterns, "auth"), namespace="auth")),
    path("admin/", admin.site.urls),
]

if settings.DEBUG:
    urlpatterns.insert(0, path("__debug__/", include("debug_toolbar.urls")))
