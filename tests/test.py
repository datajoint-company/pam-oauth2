# docker exec -it pam-oauth2_app_1 python3 test.py
import os
import pam


## simple test for user:password
p = pam.pam()
print(f"reason: {p.reason}")
response = p.authenticate(
    os.getenv("DJ_AUTH_USER"), os.getenv("DJ_AUTH_PASSWORD"), service="oidc"
)
print(f"Authenticated? {response}")
print(f"reason: {p.reason}")

response = p.authenticate(
    os.getenv("DJ_AUTH_USER"), os.getenv("DJ_AUTH_PASSWORD"), service="oidc"
)
print(f"Authenticated? {response}")
print(f"reason: {p.reason}")


## simple test for user:token
p = pam.pam()
print(
    p.authenticate(
        os.getenv("DJ_AUTH_USER"), os.getenv("DJ_AUTH_TOKEN"), service="oidc"
    )
)

print(
    p.authenticate(
        os.getenv("DJ_AUTH_USER"), os.getenv("DJ_AUTH_TOKEN"), service="oidc"
    )
)
