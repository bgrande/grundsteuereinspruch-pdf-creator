#!/bin/bash
curl -X POST \
  http://0.0.0.0:8000/html \
  -H 'Content-Type: application/json' \
  -d @./payload-example-all.json \
  & \
  browse "http://0.0.0.0:8000/formresult?subid=pdxrvZ&email=kontakt@bgrande.de&resp=2EReNj"