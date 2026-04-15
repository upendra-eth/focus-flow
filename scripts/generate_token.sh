#!/bin/bash
# Quick utility to get a dev auth token
curl -s -X POST http://localhost:8080/api/v1/auth/device \
  -H "Content-Type: application/json" \
  -d '{"device_id": "dev-test-device-001"}' | python3 -m json.tool
