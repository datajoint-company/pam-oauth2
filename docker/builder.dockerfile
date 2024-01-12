FROM mcr.microsoft.com/devcontainers/rust:1-bullseye
RUN \
	apt-get update && \
	apt-get install musl-tools libssl-dev pkg-config build-essential libpam0g-dev libpam0g gdb git -y
WORKDIR /tmp/pam-oauth2
COPY pam-oidc /tmp/pam-oauth2/pam-oidc
WORKDIR /tmp/pam-oauth2/pam-oidc
RUN cargo build && cp target/debug/libpam_oidc.so /tmp/pam-oauth2/libpam_oidc.so
