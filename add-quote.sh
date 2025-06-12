#!/bin/sh

PW=$(cat secrets/reg_password.txt)
if [ -z "$PW" ]; then
    echo "Error: secrets/reg_password.txt is empty or not found."
    exit 1
fi

CREDS="{
  \"email\": \"alex.osorio@example.com\",
  \"full_name\": \"Alex Osorio Trujillo\",
  \"password\": \"$PW\"
}"

echo "Registering to get access token..."
ACCESS_TOKEN=$(curl -s -X POST -H "Content-type: application/json" \
      -d "$CREDS" \
      http://localhost:3000/api/v1/register | jq .access_token | sed 's/"//g')

if [ -z "$ACCESS_TOKEN" ] || [ "$ACCESS_TOKEN" = "null" ]; then
    echo "Error: Failed to get access token. Check registration password and server logs."
    exit 1
fi

echo "Access token received."

QUOTE='{
  "id": "shaq-0",
  "whos_there": "Shaquille O'Neal",
  "answer_who": "I'm tired of hearing about money, money, money, money, money. I just want to play the game, drink Pepsi, wear Reebok.",
  "source": "https://www.brainyquote.com/quotes/shaquille_oneal_129496",
  "tags": [
    "lakers",
    "heat",
    "magic"
  ]
}'

echo "Attempting to add new quote..."
curl -X POST -H "Content-type: application/json" \
     -H "Authorization: Bearer $ACCESS_TOKEN" \
     -d "$QUOTE" http://localhost:3000/api/v1/add-quote

echo "\nScript finished."
