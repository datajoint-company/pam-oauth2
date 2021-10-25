# 131MB
# FROM rust:alpine
# 185 MB
# FROM rust:slim-stretch
# 196 MB
# FROM rust:slim-buster
# 425 MB
FROM rust:buster
# 438 MB
# FROM rust:stretch

# RUN \
#     curl -sSOL https://github.com/cdr/code-server/releases/download/v3.3.1/code-server_3.3.1_amd64.deb && \
#     dpkg -i code-server_3.3.1_amd64.deb

RUN \
    export uid=1000 gid=0 && \
    mkdir -p /home/rust_dev && \
    echo "rust_dev:x:${uid}:${gid}:Developer,,,:/home/rust_dev:/bin/sh" >> /etc/passwd && \
    # echo "dja:x:${uid}:" >> /etc/group && \
    chown ${uid}:${gid} -R /home/rust_dev

RUN \
    # apk add gdb git && \
    apt-get update && apt-get install gdb git -y && \
    mkdir -p /workspace/pam-rs/pam-http/target/release
    # && \
    # gdbserver :2345 ./target/debug/app

RUN \
    ln -s /lib/x86_64-linux-gnu/libpam.so.0 /lib/x86_64-linux-gnu/libpam.so && \
    apt-get install python3-pip  -y && \
    pip3 install python-pam


USER rust_dev
ENV USER rust_dev
ENV HOME /home/rust_dev


# COPY ./pam-oidc/target/release/libpam_oidc.so /workspace/pam-oidc/target/release/libpam_oidc.so
WORKDIR /workspace