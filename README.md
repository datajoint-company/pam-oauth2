# LibPAM OIDC

![GitHub Release](https://img.shields.io/github/v/release/datajoint-company/pam-oauth2)
![build](https://img.shields.io/github/actions/workflow/status/datajoint-company/pam-oauth2/release.yaml)

This repository contains a Pluggable Authentication Module (PAM) to allow authentication against a central OIDC provider.

## Deploy Instructions

1. Acquire (see the [releases](https://github.com/datajoint-company/pam-oauth2/releases) page) or build (see below) the appropriate `libpam_oidc.so` dynamic clib binary for your platform that provides the PAM interface to authenticate via an OIDC provider.
   1. The `libpam_oidc_gnu.so` binary is built for GNU/Linux distributions and dynamically links to the OS's glibc library.
   2. The `libpam_oidc_musl.so` binary is built for GNU/Linux distributions and statically links to the MUSL library. It sacrifices speed for portability.
2. Copy `libpam_oidc.so` into the appropriate directory that your system expects new PAM modules to be loaded e.g. on some distributions of Debian, it is located in `/lib/x86_64-linux-gnu/security/`, on others it is `/usr/lib64/security/`.
   1. Use `ldconfig -p | grep pam` to find the directory on your distribution.
3. Create a service config file within the directory that your system expects for PAM. For example, on Debian, it is located in `/etc/pam.d/`. We can create a service/file at `/etc/pam.d/oidc` with the following contents (note the argument in the 1st line should be the path where `pam_oidc`'s config will be located):

   ```text
   auth sufficient libpam_oidc.so /etc/datajoint/libpam_oidc.yaml
   account optional libpam_oidc.so
   ```

   See [service_example](./config/service_example) for more info.

4. In the path provided to the service config, create a config file for `pam_oidc`. See [libpam_oidc_example.yaml](./config/libpam_oidc_example.yaml) for more info.
5. Configure your PAM-compatible application/service to point to the `oidc` service we just created. For example, we can [configure Percona MySQL](https://docs.percona.com/percona-server/8.0/pam-plugin.html) to use PAM. See [MySQL testing](#3-mysql-tests) for an example.

## Developer Instructions

Since v0.1.5, the test and build have been moved to a [Docker Compose](./docker-compose.yml) file.

### Build

Since v0.1.5, releases are built in the [`builder` service](./docker-compose.yml). See the [`builder` Dockerfile](./docker/builder.dockerfile) for details. Building with Docker Compose requires a `.env` file at the repository root (it can be empty). We can build the binaries for targets `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`, respectively:

```bash
docker compose up --build builder
mkdir -p pam-oidc/bin
docker compose cp builder:/tmp/pam-oauth2/libpam_oidc_gnu.so ./pam-oidc/bin/
docker compose cp builder:/tmp/pam-oauth2/libpam_oidc_musl.so ./pam-oidc/bin/
docker compose down
```

### Testing

There are three types of tests:

1. Unit tests
2. PAM integration tests
3. MySQL tests

Some testing modes use [Docker Compose](https://docs.docker.com/compose/).
The ones that do require a `.env` file defining environment variables for the [`percona` service](./docker-compose.yml):

```bash
DJ_AUTH_USER=
DJ_AUTH_PASSWORD=
DJ_AUTH_TOKEN=
```

#### 1. Unit Tests

Unit test development is _WIP_.
Unit tests are written in Rust and only test the Rust code. They do not test the PAM integration.
Run these tests using `cargo test`.

#### 2. PAM Integration Tests

These integration tests cover the PAM integration, but none of the services that use PAM such as SSH or MySQL.
In other words, they test all but the last step in the [Deploy Instructions](#deploy-instructions).
These tests run the [`test.py`](./tests/test.py) script in the [`percona` service](./docker-compose.yml), using the [`python-pam`](https://pypi.org/project/python-pam/) library to simulate PAM calls.
To run these tests, create the `.env` file as mentioned previously, then run:

```bash
docker compose run --build percona python3 /opt/test.py
# dkc run -it percona python3 /opt/test.py
# [+] Creating 1/0
#  ✔ Container pam-oauth2-builder  Created                                                                                              0.0s
# [+] Running 1/1
#  ✔ Container pam-oauth2-builder  Started                                                                                              0.4s
# Authenticated (pam_unix)? True
# Reason (pam_unix): Success
# Authenticating with DJ_AUTH_USER='demouser'
# [2024-01-17 03:51:23.334][pam-oidc][0.1.4][INFO][656398155]: Auth detected. Proceeding...
# [2024-01-17 03:51:23.335][pam-oidc][0.1.4][INFO][656398155]: Inputs read.
# [2024-01-17 03:51:23.335][pam-oidc][0.1.4][INFO][656398155]: Check as password.
# [2024-01-17 03:51:23.651][pam-oidc][0.1.4][INFO][656398155]: Verifying token.
# [2024-01-17 03:51:23.896][pam-oidc][0.1.4][INFO][656398155]: Auth success!
# Authenticated (oidc user:pass)? True
# Reason (oidc user:pass): Success
# Authenticating with DJ_AUTH_USER='demouser'
# [2024-01-17 03:51:23.897][pam-oidc][0.1.4][INFO][656398155]: Auth detected. Proceeding...
# [2024-01-17 03:51:23.897][pam-oidc][0.1.4][INFO][656398155]: Inputs read.
# [2024-01-17 03:51:23.897][pam-oidc][0.1.4][INFO][656398155]: Check as token.
# [2024-01-17 03:51:23.897][pam-oidc][0.1.4][INFO][656398155]: Verifying token.
# [2024-01-17 03:51:24.137][pam-oidc][0.1.4][INFO][656398155]: Auth success!
# Authenticated (oidc user:token)? True
# Reason (oidc user:token): Success
```

The `test.py` script will try authenticating in three ways:

1. Using the `pam_unix` module as user `ap_user`. This checks that PAM is working in general.
2. Using the `oidc` module as user `DJ_AUTH_USER` with password `DJ_AUTH_PASSWORD`. This checks that the `pam_oidc` module is working with the password flow.
3. Using the `oidc` module as user `DJ_AUTH_USER` with token `DJ_AUTH_TOKEN`. This checks that the `pam_oidc` module is working with the token flow.

#### 3. MySQL Tests

These tests cover the PAM integration with MySQL.
Create the `.env` file and issue the following commands, replacing `demouser` with the value of `DJ_AUTH_USER` in the `.env` file:

```bash
docker compose up --build -d percona
# Wait until service is healthy, then create demouser
docker compose exec percona mysql -hlocalhost -uroot -ppassword -e "CREATE USER 'demouser'@'%' IDENTIFIED WITH auth_pam AS 'oidc';"
# Check if the auth_pam plugin is enabled in Percona
docker compose exec percona mysql -hlocalhost -uroot -ppassword -e "SHOW PLUGINS;" | grep auth_pam
# auth_pam        ACTIVE  AUTHENTICATION  auth_pam.so     GPL
# Login as demouser
docker compose exec percona mysql -hlocalhost -udemouser -p'password_or_token_in_dot_env' -e "SELECT 1;"
# +---+
# | 1 |
# +---+
# | 1 |
# +---+
docker compose down
```

Successful login will return a table with a single row containing the value `1`.
This indicates that the `pam_oidc` module is working with MySQL.

## Old Developer Instructions

Below are the old instructions for building and testing the `pam_oidc` module.

<details>
<summary>Click to expand</summary>

### Build

```bash
cd ./pam-oidc && cargo build; cd ..  # DEBUG
cd ./pam-oidc && cargo build --release; cd ..  # PROD
```

### Validate PAM with test cases

Create `.env` file in the root directory with the following:
```
DJ_AUTH_USER=
DJ_AUTH_PASSWORD=
DJ_AUTH_TOKEN=
```
See tests in `tests` subdirectory. The header comment gives hints how to run them.

### Testing `pam_unix` Plugin in Percona

Following [Percona blog post](https://www.percona.com/blog/getting-percona-pam-to-work-with-percona-server-its-client-apps/):

```console
❯ alias dkc='docker compose'
❯ dkc up --build -d percona
❯ dkc exec -it percona mysql -hlocalhost -uroot -ppassword -e "SHOW PLUGINS;" | grep auth_pam
auth_pam        ACTIVE  AUTHENTICATION  auth_pam.so     GPL
❯ dkc exec -it percona mysql -hlocalhost -uroot -ppassword
mysql: [Warning] Using a password on the command line interface can be insecure.
Welcome to the MySQL monitor.  Commands end with ; or \g.
Your MySQL connection id is 19
Server version: 8.0.34-26 Percona Server (GPL), Release 26, Revision 0fe62c85

Copyright (c) 2009-2023 Percona LLC and/or its affiliates
Copyright (c) 2000, 2023, Oracle and/or its affiliates.

Oracle is a registered trademark of Oracle Corporation and/or its
affiliates. Other names may be trademarks of their respective
owners.

Type 'help;' or '\h' for help. Type '\c' to clear the current input statement.

mysql> CREATE USER ap_user IDENTIFIED WITH auth_pam;
Query OK, 0 rows affected (0.04 sec)

mysql> DELETE FROM mysql.user WHERE USER='';
Query OK, 0 rows affected (0.00 sec)

mysql> FLUSH PRIVILEGES;
Query OK, 0 rows affected (0.02 sec)

mysql>
Bye
❯ dkc exec -it percona mysql -hlocalhost -uap_user -ppassword
# Success
```

### Start

To do in local folder
`cargo init`


### Debug

`cargo run`

### Build (debug, prod)

`cargo build`
`cargo build --release`


### test case (needed to install gcc, g++, openssl, libressl-dev, pkgconfig, OPENSSL_DIR=/etc/ssl)

*as root

apk add g++ libressl-dev
apt-get install libssl-dev pkg-config -y
apt-get install musl-tools -y

apt-get install libssl-dev pkg-config build-essential libpam0g-dev libpam0g -y

*as user

cd /workspace/pam-oidc

cargo build

echo shh | PAM_TYPE=auth PAM_USER=raphael ./pam_oidc/target/release/pam_oidc ./sample.yaml


### cross-compile

rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-unknown-linux-musl
rustup show
cargo build --target x86_64-unknown-linux-musl --features vendored
cargo build --release --target x86_64-unknown-linux-musl

### testing (current on 07/01/21)

cp pam-oidc/test /etc/pam.d/
cp pam-oidc/target/debug/libpam_oidc.so /lib/x86_64-linux-gnu/security/
python3 /workspace/test.py

</details>