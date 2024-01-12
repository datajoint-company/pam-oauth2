"""
The purpose of this script is to ingest secrets in config/libpam_oidc.yaml
and .env files and test authentication against the KeyCloak OIDC provider.
"""

import sys
import yaml
from typing import Dict
from oic.oic.message import ProviderConfigurationResponse
from oic.oic import Client
from oic.utils.authn.client import CLIENT_AUTHN_METHOD
from oic.oic.message import RegistrationResponse


# CONFIG_PATH = 'config/libpam_oidc.yaml'
CONFIG_PATH = '/etc/datajoint/libpam_oidc.yaml'


def read_config(path=CONFIG_PATH) -> Dict:
    with open(path, 'r') as f:
        config = yaml.safe_load(f)
    return config


def auth_flow(client):
    pass

def main():
    """
        "issuer": SINGLE_REQUIRED_STRING,
        "authorization_endpoint": SINGLE_REQUIRED_STRING,
        "token_endpoint": SINGLE_OPTIONAL_STRING,
        "userinfo_endpoint": SINGLE_OPTIONAL_STRING,
        "jwks_uri": SINGLE_REQUIRED_STRING,
        "registration_endpoint": SINGLE_OPTIONAL_STRING,
        "scopes_supported": OPTIONAL_LIST_OF_STRINGS,
        "response_types_supported": REQUIRED_LIST_OF_STRINGS,
        "response_modes_supported": OPTIONAL_LIST_OF_STRINGS,
        "grant_types_supported": OPTIONAL_LIST_OF_STRINGS,
        "acr_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "subject_types_supported": REQUIRED_LIST_OF_STRINGS,
        "id_token_signing_alg_values_supported": REQUIRED_LIST_OF_STRINGS,
        "id_token_encryption_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "id_token_encryption_enc_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "userinfo_signing_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "userinfo_encryption_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "userinfo_encryption_enc_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "request_object_signing_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "request_object_encryption_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "request_object_encryption_enc_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "token_endpoint_auth_methods_supported": OPTIONAL_LIST_OF_STRINGS,
        "token_endpoint_auth_signing_alg_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "display_values_supported": OPTIONAL_LIST_OF_STRINGS,
        "claim_types_supported": OPTIONAL_LIST_OF_STRINGS,
        "claims_supported": OPTIONAL_LIST_OF_STRINGS,
        "service_documentation": SINGLE_OPTIONAL_STRING,
        "claims_locales_supported": OPTIONAL_LIST_OF_STRINGS,
        "ui_locales_supported": OPTIONAL_LIST_OF_STRINGS,
        "claims_parameter_supported": SINGLE_OPTIONAL_BOOLEAN,
        "request_parameter_supported": SINGLE_OPTIONAL_BOOLEAN,
        "request_uri_parameter_supported": SINGLE_OPTIONAL_BOOLEAN,
        "require_request_uri_registration": SINGLE_OPTIONAL_BOOLEAN,
        "op_policy_uri": SINGLE_OPTIONAL_STRING,
        "op_tos_uri": SINGLE_OPTIONAL_STRING,
        "check_session_iframe": SINGLE_OPTIONAL_STRING,
        "end_session_endpoint": SINGLE_OPTIONAL_STRING,
        "frontchannel_logout_supported": SINGLE_OPTIONAL_BOOLEAN,
        "frontchannel_logout_session_supported": SINGLE_OPTIONAL_BOOLEAN,
        "backchannel_logout_supported": SINGLE_OPTIONAL_BOOLEAN,
        "backchannel_logout_session_supported": SINGLE_OPTIONAL_BOOLEAN,
    }
    c_default = {
        "version": "3.0",
        "token_endpoint_auth_methods_supported": ["client_secret_basic"],
        "claims_parameter_supported": False,
        "request_parameter_supported": False,
        "request_uri_parameter_supported": True,
        "require_request_uri_registration": False,
        "grant_types_supported": ["authorization_code", "implicit"],
        "frontchannel_logout_supported": False,
        "frontchannel_logout_session_supported": False,
        "backchannel_logout_supported": False,
        "backchannel_logout_session_supported": False,
    }
    """
    client = Client(client_authn_method=CLIENT_AUTHN_METHOD)
    config = read_config()
    conn_info = {
        'issuer': "/".join(config['url.auth'].split('/')[:-1]) + '/',
        'authorization_endpoint': config['url.auth'],
        'token_endpoint': config['url.token'],
        'userinfo_endpoint': config['url.userinfo'],
        # 'jwks_uri': config['url.jwks'],
    }
    print(f"Using {conn_info=}")
    op_info = ProviderConfigurationResponse(**conn_info)
    client.handle_provider_config(op_info, op_info['issuer'])

    # Likewise, if the client registration has been done out-of-band:
    info = {"client_id": config['client.id'], "client_secret": config['client.secret']}
    client_reg = RegistrationResponse(**info)
    client.store_registration_info(client_reg)

    # Auth flow
    from oic import rndstr
    from oic.utils.http_util import Redirect

    session = dict()
    session["state"] = rndstr()
    session["nonce"] = rndstr()
    args = {
        "client_id": client.client_id,
        "response_type": "code",
        "scope": ["openid"],
        "nonce": session["nonce"],
        "redirect_uri": "https://fakeservices.datajoint.io",
        "state": session["state"]
    }
    auth_req = client.construct_AuthorizationRequest(request_args=args)
    login_url = auth_req.request(client.authorization_endpoint)


if __name__ == '__main__':
    main(*sys.argv[1:])