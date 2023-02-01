#!/bin/bash
curl -X POST \
  https://pdf-creator-shuttle.shuttleapp.rs/html \
  -H 'Content-Type: application/json' \
  -d @./payload-example.json \
  & \
  browse "https://pdf-creator-shuttle.shuttleapp.rs/formresult?subid=jZypz1&email=kontakt@bgrande.de&resp=2EReNj"