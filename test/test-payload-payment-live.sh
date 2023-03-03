#!/bin/bash
curl -X POST \
  https://app.grundsteuereinspruch.online/html \
  -H 'Content-Type: application/json' \
  -d @./test/payload-example-payment.json \
  -v \
  -o test-payload.log \
  & \
  browse "https://app.grundsteuereinspruch.online/formresult?subid=VjYeNM&email=kontakt@bgrande.de&resp=2EReNj"