#!/bin/bash

# set -a && . .env && ./tests/test.sh mariadb && set +a
# set -a && . .env && ./tests/test.sh percona && set +a

mariadb() {
    set -e
    ROOT_PASSWORD=simple
    docker rm -f database
    docker run --name database -de MYSQL_ROOT_PASSWORD=${ROOT_PASSWORD} mariadb:10.7 # does not work with latest and non-v1
    until docker exec -it database mysql -h 127.0.0.1 -uroot -p${ROOT_PASSWORD} -e "SELECT 1;" 1>/dev/null
    do
        echo waiting...
        sleep 5
    done
    docker exec -it database mysql -uroot -p${ROOT_PASSWORD} -e "INSTALL SONAME 'auth_pam_v1';"
    docker cp ./config/service_example database:/etc/pam.d/oidc
    docker cp ./pam-oidc/target/debug/libpam_oidc.so database:/lib/x86_64-linux-gnu/security/libpam_oidc.so
    docker exec -it database mkdir /etc/datajoint
    docker cp ./config/libpam_oidc.yaml database:/etc/datajoint/
    docker exec -it database mysql -uroot -p${ROOT_PASSWORD} -e "CREATE USER '${DJ_AUTH_USER}'@'%' IDENTIFIED VIA pam USING 'oidc';"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -pdeny -e "SELECT 'delegated to oidc' as login;"
}

percona() {
    set -e
    ROOT_PASSWORD=simple
    docker rm -f database
    docker run --name database -de MYSQL_ROOT_PASSWORD=${ROOT_PASSWORD} --entrypoint bash percona:8 -c "echo 'plugin_load_add = auth_pam.so' >> /etc/my.cnf && /docker-entrypoint.sh mysqld"
    until docker exec -it database mysql -h 127.0.0.1 -uroot -p${ROOT_PASSWORD} -e "SELECT 1;" 1>/dev/null
    do
        echo waiting...
        sleep 5
    done
    docker cp ./config/service_example database:/etc/pam.d/oidc
    docker cp ./pam-oidc/target/debug/libpam_oidc.so database:/usr/lib64/security/libpam_oidc.so
    docker exec -itu root database mkdir /etc/datajoint
    docker cp ./config/libpam_oidc.yaml database:/etc/datajoint/
    docker exec -it database mysql -uroot -p${ROOT_PASSWORD} -e "CREATE USER '${DJ_AUTH_USER}'@'%' IDENTIFIED WITH auth_pam AS 'oidc';"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -p${DJ_AUTH_PASSWORD} -e "SELECT 'delegated to oidc' as login;"
    docker exec -it database mysql -h 127.0.0.1 -u${DJ_AUTH_USER} -pdeny -e "SELECT 'delegated to oidc' as login;"
}

