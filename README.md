# Rust Demo

## Deploy Instructions

1. Acquire (see the [releases](https://github.com/vathes/pam-oauth2/releases) page) or build (see below) the appropriate `libpam_oidc.so` dynamic clib binary for your platform that provides the PAM interface to authenticate via an OIDC provider.
1. Copy `libpam_oidc.so` into the appropriate directory that your system expects new modules to be loaded e.g. on Debian, it is located in `/lib/x86_64-linux-gnu/security/`.
1. Create a service config file within the directory that your system expects for PAM e.g. on Debian, it is located in `/etc/pam.d/`. We can for instance create a service/file called `oidc` with the following contents (note the argument in the 1st line should be the path where `pam_oidc`'s config will be located):

   ```text
   auth sufficient libpam_oidc.so /etc/datajoint/libpam_oidc.yaml
   account optional libpam_oidc.so
   ```

   See [service_example](./config/service_example) for more info.

1. In the path provided to the service config, create a config file for `pam_oidc`. See [libpam_oidc_example.yaml](./config/libpam_oidc_example.yaml) for more info.
1. Configure your PAM-compatible application/service to point to the `oidc` service we just created. For a few examples, see [test.sh](./tests/test.sh).

## Developer Instructions

### Build

```bash
cd ./pam-oidc && cargo build; cd ..  # DEBUG
cd ./pam-oidc && cargo build --release; cd ..  # PROD
```

### Validate PAM with test cases

See tests in `tests` subdirectory. The header comment gives hints how to run them.

## --- Old Notes ---

### Start

To do in local folder
`cargo init`


### Debug

`cargo run`

### Build (debug, prod)

`cargo build`
`cargo build --release`


### test case (needed to install gcc, g++, openssl, libressl-dev, pkgconfig, OPENSSL_DIR=/etc/ssl)

*as root

apk add g++ libressl-dev
apt-get install libssl-dev pkg-config -y
apt-get install musl-tools -y

apt-get install libssl-dev pkg-config build-essential libpam0g-dev libpam0g -y

*as user

cd /workspace/pam-oidc

cargo build

echo shh | PAM_TYPE=auth PAM_USER=raphael ./pam_oidc/target/release/pam_oidc ./sample.yaml


## cross-compile

rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup show
cargo build --target x86_64-unknown-linux-musl --features vendored
cargo build --release --target x86_64-unknown-linux-musl

## testing (current on 07/01/21)

cp pam-oidc/test /etc/pam.d/
cp pam-oidc/target/debug/libpam_oidc.so /lib/x86_64-linux-gnu/security/
python3 /workspace/test.py