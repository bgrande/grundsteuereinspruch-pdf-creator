#!/bin/bash
curl -X POST \
  https://app.grundsteuereinspruch.online/html \
  -H 'Content-Type: application/json' \
  -d @./test/payload-example.json \
  & \
  browse "https://app.grundsteuereinspruch.online/formresult?subid=jZypz1&email=kontakt@bgrande.de&resp=2EReNj"