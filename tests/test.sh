#!/bin/bash

# Usage:
# ./tests/test.sh '<demouser_password>'

docker compose up --build -d --wait percona
docker compose exec percona mysql -hlocalhost -uroot -ppassword -e "CREATE USER 'demouser'@'%' IDENTIFIED WITH auth_pam AS 'mysql-any-password';"
docker compose exec percona mysql -hlocalhost -udemouser -p"$1" --enable-cleartext-plugin -e "SELECT 1;" || echo "Failed to authenticate with real password"
docker compose exec percona mysql -hlocalhost -udemouser -p'bogus_password' --enable-cleartext-plugin -e "SELECT 1;" || echo "Failed to authenticate for bogus password"
sleep 3
docker compose logs percona
docker compose down