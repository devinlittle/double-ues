#!/usr/bin/env bash
bunx openapi-typescript "http://0.0.0.0:5252/api-docs/openapi.json" -o "./src/lib/types/chat.api.ts"
