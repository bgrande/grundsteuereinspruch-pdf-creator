#!/bin/bash

#./install_nginx.sh "app.grundsteuereinspruch.online" gseonlineadmin "app.grundsteuereinspruch.online" "nginx-http" "/home/pdfcreator/app" 8081

sleep 10

./tlscert_nginx.sh "app.grundsteuereinspruch.online" gseonlineadmin "app.grundsteuereinspruch.online"

sleep 150

./install_nginx.sh "app.grundsteuereinspruch.online" gseonlineadmin "app.grundsteuereinspruch.online" "nginx-rp" "/home/pdfcreator/static" 8081