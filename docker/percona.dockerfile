FROM datajoint/pam-oauth2-builder:latest as builder
FROM percona:8
# COPY --from=builder /tmp/pam-oauth2/libpam_oidc.so /lib/x86_64-linux-gnu/security/libpam_oidc.so
COPY --from=builder /tmp/pam-oauth2/libpam_oidc.so /usr/lib64/security/libpam_oidc.so
RUN echo 'plugin_load_add = auth_pam.so' >> /etc/my.cnf

# https://www.percona.com/blog/getting-percona-pam-to-work-with-percona-server-its-client-apps/
USER root
RUN \
	chgrp mysql /etc/shadow && \
	chmod g+r /etc/shadow && \
	useradd ap_user && \
	echo "ap_user:password" | chpasswd
RUN \
	yum -y install python3 python3-pip && \
	pip3 install python-pam
USER mysql

# https://docs.percona.com/percona-server/8.0/pam-plugin.html#installation
# COPY config/pam_unix /etc/pam.d/mysqld
# COPY config/service_example /etc/pam.d/oidc
COPY config/libpam_oidc.yaml /etc/datajoint/libpam_oidc.yaml