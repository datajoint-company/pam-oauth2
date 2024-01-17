# [DEV-419](https://datajoint.atlassian.net/browse/DEV-419): Improve Logging

In DataJoint Works QA deployment of Percona on k8s, each line of the PAM OIDC logs starts with `{"` and shows every `\n` character explicitly, instead of printing each line individually.

## Work Plan

- [x] Attempt to reproduce log formatting on Docker compose
- [ ] Try [`x86_64-unknown-netbsd`](https://doc.rust-lang.org/nightly/rustc/platform-support.html) target
    - The thought process is: v0.1.4 binary was named `libpam_oidc_linux_amd64.so`, which indicates an AMD64 target.
    - We're currently targeting x86_64 arch, and could try targeting the NetBSD/AMD64 arch instead. Not sure why this would resolve logging.
- [x] Investigate [`logStructured: false`](https://github.com/yambottle/dj-gitops/blob/3c08d41875aa54664cfb171af34f6edb2ab3b598/applications/k8s/deployments/percona-with-helm/percona-op-values.yaml#L58)
    - Possibly [structured logging](https://kubernetes.io/docs/concepts/cluster-administration/system-logs/#structured-logging). This is only supported as of k8s v1.23, which is beyond our current version I believe.

## Steps to Reproduce

### Test v0.1.4 Logging in Prod Percona

As a control group, I fetched the v0.1.4 binary and checked the Percona log format.

```dockerfile
# docker/prod-percona.dockerfile
# ...
ADD https://github.com/datajoint-company/pam-oauth2/releases/download/0.1.4/libpam_oidc_linux_amd64.so /usr/lib64/security/libpam_oidc.so
# ...
```

```bash
alias dkc="docker compose"
cp percona.dockerfile prod-percona.dockerfile
dkc up --build prod-percona
dkc exec -it prod-percona mysql -hlocalhost -uroot -p'password' -e "SELECT 1;"
dkc exec -it prod-percona mysql -hlocalhost -uroot -ppassword -e "INSTALL PLUGIN auth_pam SONAME 'auth_pam.so';"
dkc exec -it prod-percona mysql -hlocalhost -uroot -ppassword -e "SHOW PLUGINS;"
dkc exec -it prod-percona mysql -hlocalhost -uroot -ppassword -e "CREATE USER 'ap_user'@'%' IDENTIFIED WITH auth_pam;"
dkc exec -it prod-percona mysql -hlocalhost -uroot -ppassword -e "CREATE USER 'demouser'@'%' IDENTIFIED WITH auth_pam AS 'oidc';"
dkc exec -it prod-percona mysql -hlocalhost -uap_user -ppassword -e "SELECT 1;"
dkc exec -it prod-percona mysql -hlocalhost -udemouser -p'<password_from_dot_env>' -e "SELECT 1;"
```

`prod-percona` logs:

```console
# ...
pam-oauth2-prod-percona  | [2024-01-17 20:52:39.294][pam-oidc][0.1.4][INFO][3207253868]: Auth detected. Proceeding...
pam-oauth2-prod-percona  | [2024-01-17 20:52:39.294][pam-oidc][0.1.4][INFO][3207253868]: Inputs read.
pam-oauth2-prod-percona  | [2024-01-17 20:52:39.294][pam-oidc][0.1.4][INFO][3207253868]: Check as password.
pam-oauth2-prod-percona  | [2024-01-17 20:52:39.757][pam-oidc][0.1.4][INFO][3207253868]: Verifying token.
pam-oauth2-prod-percona  | [2024-01-17 20:52:40.035][pam-oidc][0.1.4][INFO][3207253868]: Auth success!
pam-oauth2-prod-percona  | 2024-01-17T20:52:40.036162Z 36 [ERROR] [MY-000000] [Server] Plugin auth_pam reported: 'Unable to obtain the passwd entry for the user 'demouser'.'
```

Log format is as expected.

### Test v0.1.5 Logging in Prod Percona

Instead of fetching the v0.1.4 binary:

```dockerfile
# docker/prod-percona.dockerfile
# ...
ADD https://github.com/datajoint-company/pam-oauth2/releases/download/v0.1.5/libpam_oidc_musl.so /usr/lib64/security/libpam_oidc.so
# ...
```

```bash
# Same docker compose commands as above
```

`prod-percona` logs look exactly the same using v0.1.5:

```console
# ...
pam-oauth2-prod-percona  | [2024-01-17 21:12:06.606][pam-oidc][0.1.5][INFO][1619253419]: Auth detected. Proceeding...
pam-oauth2-prod-percona  | [2024-01-17 21:12:06.606][pam-oidc][0.1.5][INFO][1619253419]: Inputs read.
pam-oauth2-prod-percona  | [2024-01-17 21:12:06.606][pam-oidc][0.1.5][INFO][1619253419]: Check as password.
pam-oauth2-prod-percona  | [2024-01-17 21:12:06.896][pam-oidc][0.1.5][INFO][1619253419]: Verifying token.
pam-oauth2-prod-percona  | [2024-01-17 21:12:07.129][pam-oidc][0.1.5][INFO][1619253419]: Auth success!
pam-oauth2-prod-percona  | 2024-01-17T21:12:07.129555Z 31 [ERROR] [MY-000000] [Server] Plugin auth_pam reported: 'Unable to obtain the passwd entry for the user 'demouser'.'
```

Log format is the same. When I pass the wrong password for `demouser`:

```console
# ...
pam-oauth2-prod-percona  | [2024-01-17 21:19:33.400][pam-oidc][0.1.5][INFO][1619253419]: Auth detected. Proceeding...
pam-oauth2-prod-percona  | [2024-01-17 21:19:33.400][pam-oidc][0.1.5][INFO][1619253419]: Inputs read.
pam-oauth2-prod-percona  | [2024-01-17 21:19:33.400][pam-oidc][0.1.5][INFO][1619253419]: Check as password.
pam-oauth2-prod-percona  | [2024-01-17 21:19:33.701][pam-oidc][0.1.5][INFO][1619253419]: Wrong password provided.
```

