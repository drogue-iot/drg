#!/bin/bash

drg login "$DROGUE_SANDBOX_URL" -c test --access-token "$DROGUE_SANDBOX_USERNAME":"$DROGUE_SANDBOX_ACCESS_KEY"

for a in $(drg get apps | awk 'NR>2 {print $1}')
do
   drg delete app "$a"
done