# Rust Demo

## Start

To do in local folder
`cargo init`


## Debug

`cargo run`

## Build (debug, prod)

`cargo build`
`cargo build --release`


## test case (needed to install gcc, g++, openssl, libressl-dev, pkgconfig, OPENSSL_DIR=/etc/ssl)

*as root

apk add g++ libressl-dev
apt-get install libssl-dev pkg-config -y
apt-get install musl-tools -y

apt-get install libssl-dev pkg-config build-essential libpam0g-dev libpam0g -y

*as user

cd /workspace/app

cargo build

echo shh | PAM_TYPE=auth PAM_USER=raphael ./pam_oidc/target/release/pam_oidc ./sample.yaml


# cross-compile

rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup show
cargo build --target x86_64-unknown-linux-musl --features vendored
cargo build --release --target x86_64-unknown-linux-musl

# testing

cp pam-oidc/test /etc/pam.d/
cp pam-oidc/target/debug/libpam_oidc.so /lib/x86_64-linux-gnu/security/