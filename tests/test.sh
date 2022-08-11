#!/bin/bash

# set -a && . .env && ./tests/test.sh mariadb && set +a
# set -a && . .env && ./tests/test.sh percona && set +a

mariadb() {
    set -e
    docker rm -f database
    docker run --name database -p 3306:3306 -de MYSQL_ROOT_PASSWORD=simple mariadb:10.7 # does not work with latest and non-v1
    until docker exec -it database mysql -h 127.0.0.1 -uroot -psimple -e "SELECT 1;" 1>/dev/null
    do
        echo waiting...
        sleep 5
    done
    docker exec -it database mysql -uroot -psimple -e "INSTALL SONAME 'auth_pam_v1';"
    docker cp /home/rguzman/Downloads/oidc database:/etc/pam.d/oidc
    docker cp /github/pam-oauth2/pam-oidc/target/debug/libpam_oidc.so database:/lib/x86_64-linux-gnu/security/libpam_oidc.so
    docker exec -it database mkdir /etc/datajoint
    docker cp /home/rguzman/Downloads/libpam_oidc.yaml database:/etc/datajoint/
    docker exec -it database mysql -uroot -psimple -e "CREATE USER '${DJ_AUTH_USER}'@'%' IDENTIFIED VIA pam USING 'oidc';"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -pdeny -e "SELECT 'delegated to oidc' as login;"
}

percona() {
    set -e
    docker rm -f database
    docker run --name database -p 3306:3306 -de MYSQL_ROOT_PASSWORD=simple percona:8
    until docker exec -it database mysql -h 127.0.0.1 -uroot -psimple -e "SELECT 1;" 1>/dev/null
    do
        echo waiting...
        sleep 5
    done
    docker exec -it database mysql -uroot -psimple -e "INSTALL PLUGIN auth_pam SONAME 'auth_pam.so';"
    docker cp /home/rguzman/Downloads/oidc database:/etc/pam.d/mysqld
    docker cp /github/pam-oauth2/pam-oidc/target/debug/libpam_oidc.so database:/usr/lib64/security/libpam_oidc.so
    docker exec -itu root database mkdir /etc/datajoint
    docker cp /home/rguzman/Downloads/libpam_oidc.yaml database:/etc/datajoint/
    docker exec -it database mysql -uroot -psimple -e "CREATE USER '${DJ_AUTH_USER}'@'%' IDENTIFIED WITH auth_pam;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -pdeny -e "SELECT 'delegated to oidc' as login;"
}

"$@"