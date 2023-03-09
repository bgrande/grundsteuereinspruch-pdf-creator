#!/bin/bash
curl -X POST \
  https://app.grundsteuereinspruch.online/html \
  -H 'Content-Type: application/json' \
  -d @./test/payload-example-all.json \
  & \
  browse "https://app.grundsteuereinspruch.online/formresult?subid=pdxrvZ&email=kontakt@bgrande.de&resp=2EReNj"