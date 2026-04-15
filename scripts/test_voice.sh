#!/bin/bash
# Test voice upload endpoint
# Usage: ./test_voice.sh <audio_file> <jwt_token>
AUDIO_FILE="${1:?Usage: $0 <audio_file> <jwt_token>}"
TOKEN="${2:?Usage: $0 <audio_file> <jwt_token>}"
curl -X POST http://localhost:8080/api/v1/voice/upload \
  -H "Authorization: Bearer $TOKEN" \
  -F "audio=@$AUDIO_FILE" | python3 -m json.tool
