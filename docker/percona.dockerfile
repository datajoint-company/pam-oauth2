FROM datajoint/pam-oauth2-builder:v0.1.4 as builder
FROM percona:8
USER root

# Fetch the binary from the release page
RUN \
	yum -y install wget python3 python3-pip && \
	pip3 install python-pam
RUN \
	wget https://github.com/datajoint-company/pam-oauth2/releases/download/0.1.4/libpam_oidc_linux_amd64.so && \
	cp ./libpam_oidc_linux_amd64.so /usr/lib64/security/libpam_oidc.so && \
	chmod +rx /usr/lib64/security/libpam_oidc.so && \
	ls /usr/lib64/security/

# https://www.percona.com/blog/getting-percona-pam-to-work-with-percona-server-its-client-apps/
RUN \
	chgrp mysql /etc/shadow && \
	chmod g+r /etc/shadow && \
	useradd ap_user && \
	echo "ap_user:password" | chpasswd
USER mysql

# COPY --from=builder /tmp/pam-oauth2/libpam_oidc.so /lib/x86_64-linux-gnu/security/libpam_oidc.so
# COPY --from=builder /tmp/pam-oauth2/libpam_oidc.so /usr/lib64/security/libpam_oidc.so
RUN echo 'plugin_load_add = auth_pam.so' >> /etc/my.cnf

# https://docs.percona.com/percona-server/8.0/pam-plugin.html#installation
COPY config/pam_unix /etc/pam.d/mysqld
COPY config/service_example /etc/pam.d/oidc
COPY config/libpam_oidc.yaml /etc/datajoint/libpam_oidc.yaml