FROM mcr.microsoft.com/devcontainers/rust:1.0.7-bullseye
RUN \
	apt-get update && \
	apt-get install \
		musl-tools libssl-dev pkg-config libssl-dev build-essential \
		gcc g++ openssl \
		libpam0g-dev libpam0g gdb git -y
ENV OPENSSL_DIR=/etc/ssl
ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /tmp/pam-oauth2
COPY pam-oidc /tmp/pam-oauth2/pam-oidc
WORKDIR /tmp/pam-oauth2/pam-oidc
RUN \
	rustup target add x86_64-unknown-linux-gnu && \
	rustup target add x86_64-unknown-linux-musl && \
	rustup show && \
	cargo build --release --target x86_64-unknown-linux-musl
RUN \
	cp target/x86_64-unknown-linux-musl/release/libpam_oidc.so /tmp/pam-oauth2/libpam_oidc.so
