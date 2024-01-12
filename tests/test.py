# docker compose exec -it percona python3 /opt/test.py
import os
import pam

DJ_AUTH_USER = os.getenv("DJ_AUTH_USER", "")
DJ_AUTH_PASSWORD = os.getenv("DJ_AUTH_PASSWORD", "")
DJ_AUTH_TOKEN = os.getenv("DJ_AUTH_TOKEN", "")

# Test pam_unix via mysqld module
p = pam.pam()
response = p.authenticate(
    'ap_user', 'password',
    service='mysqld'
)
print(f"Authenticated (pam_unix)? {response}")
print(f"Reason (pam_unix): {p.reason}")

# Test oidc with user:password
p = pam.pam()
print(f"Authenticating with {DJ_AUTH_USER=}")
response = p.authenticate(
    DJ_AUTH_USER, DJ_AUTH_PASSWORD, service="oidc"
)
print(f"Authenticated (oidc user:pass)? {response}")
print(f"Reason (oidc user:pass): {p.reason}")

# Test oidc with user:token
print(f"Authenticating with {DJ_AUTH_USER=}")
response = p.authenticate(
    DJ_AUTH_USER, DJ_AUTH_TOKEN, service="oidc"
)
print(f"Authenticated (oidc user:token)? {response}")
print(f"Reason (oidc user:token): {p.reason}")
