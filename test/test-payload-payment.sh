#!/bin/bash
curl -X POST \
  http://0.0.0.0:8000/html \
  -H 'Content-Type: application/json' \
  -d @./test/payload-example-payment.json \
  -v \
  & \
  browse "http://0.0.0.0:8000/formresult?subid=VjYeNM&email=kontakt@bgrande.de&resp=2EReNj"