#!/bin/bash
curl -X POST \
  http://0.0.0.0:8000/html \
  -H 'Content-Type: application/json' \
  -d @./test/payload-example-empty-dates.json \
  & \
  browse "http://0.0.0.0:8000/formresult?subid=jZypz1&email=kontakt@bgrande.de&resp=2EReNj"