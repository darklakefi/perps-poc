#!/bin/bash

echo "Creating user..."
curl -X POST http://localhost:3000/create_user \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 123   
  }'

echo -e "\n\nDepositing funds..."
curl -X POST http://localhost:3000/deposit \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 123,  
    "amount": 10000,  
    "key": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]
  }'

echo -e "\n\nOpening position..."
curl -X POST http://localhost:3000/open_position \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": 123,  
    "direction": true,
    "entry_price": 50000,
    "notional": 1000,
    "leverage": 10,
    "initial_margin": 100
  }'

echo -e "\n\nDone!"