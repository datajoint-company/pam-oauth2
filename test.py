import os
import pam


## simple test for user:password
p = pam.pam()
print(p.authenticate(os.getenv('DJ_AUTH_USER'), os.getenv('DJ_AUTH_PASSWORD'), service='oidc'))


## simple test for user:token
# p = pam.pam()
# print(p.authenticate(os.getenv('DJ_AUTH_USER'), os.getenv('DJ_AUTH_TOKEN'), service='oidc'))
