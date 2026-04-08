#!/bin/bash

KEY="your-secret-key"
PLAYER="hyriik"
URL="http://localhost:8787/v1/players/$PLAYER"

curl -s -H "Authorization: Bearer $KEY" "$URL" | jq
