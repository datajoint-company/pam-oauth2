ARG BUILDER_TAG
FROM datajoint/pam-oauth2-builder:${BUILDER_TAG} as builder
FROM percona/percona-xtradb-cluster:8.0.29-21.1
USER root

# Fetch the binary from the release page
ADD https://github.com/datajoint-company/pam-oauth2/releases/download/v0.1.5/libpam_oidc_musl.so /usr/lib64/security/libpam_oidc.so
RUN chmod +rx /usr/lib64/security/libpam_oidc.so

# https://www.percona.com/blog/getting-percona-pam-to-work-with-percona-server-its-client-apps/
RUN \
	chgrp mysql /etc/shadow && \
	chmod g+r /etc/shadow && \
	useradd ap_user && \
	echo "ap_user:password" | chpasswd && \
	echo 'plugin_load_add = auth_pam.so' # >> /etc/my.cnf
USER mysql:mysql

# https://docs.percona.com/percona-server/8.0/pam-plugin.html#installation
# COPY --from=builder /tmp/pam-oauth2/libpam_oidc_gnu.so /usr/lib64/security/libpam_oidc.so
COPY config/pam_unix /etc/pam.d/mysqld
COPY config/service_example /etc/pam.d/oidc
