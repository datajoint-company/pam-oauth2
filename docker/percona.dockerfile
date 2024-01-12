FROM datajoint/pam-oauth2-builder:latest as builder
FROM percona:8
COPY --from=builder /tmp/pam-oauth2/libpam_oidc.so /lib/x86_64-linux-gnu/security/libpam_oidc.so
RUN echo 'plugin_load_add = auth_pam.so' >> /etc/my.cnf

# https://docs.percona.com/percona-server/8.0/pam-plugin.html#installation
COPY config/pam_unix /etc/pam.d/mysqld
COPY config/service_example /etc/pam.d/oidc
COPY config/libpam_oidc.yaml /etc/datajoint/libpam_oidc.yaml