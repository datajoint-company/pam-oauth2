FROM mcr.microsoft.com/devcontainers/rust:1.0.7-bullseye
RUN \
	apt-get update && \
	apt-get install \
		musl-tools libssl-dev pkg-config libssl-dev build-essential \
		gcc g++ openssl \
		libpam0g-dev libpam0g gdb git -y
ENV RUSTFLAGS="-C target-feature=-crt-static"
WORKDIR /tmp/pam-oauth2
COPY pam-oidc /tmp/pam-oauth2/pam-oidc
# RUN \
# 	cd pam-oidc && \
# 	rustup target add x86_64-unknown-linux-gnu && \
# 	rustup target add x86_64-unknown-linux-musl && \
# 	rustup show && \
# 	cargo build --release --target x86_64-unknown-linux-musl && \
# 	cargo build --release --target x86_64-unknown-linux-gnu && \
# 	cp target/x86_64-unknown-linux-musl/release/libpam_oidc.so /tmp/pam-oauth2/libpam_oidc_musl.so && \
# 	cp target/x86_64-unknown-linux-gnu/release/libpam_oidc.so /tmp/pam-oauth2/libpam_oidc_gnu.so
RUN \
	cd pam-oidc && \
	rustup target add x86_64-unknown-linux-gnu && \
	rustup show && \
	cargo build --target x86_64-unknown-linux-gnu && \
	cp target/x86_64-unknown-linux-gnu/debug/libpam_oidc.so /tmp/pam-oauth2/libpam_oidc_gnu.so