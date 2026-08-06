#!/bin/bash
while ! docker info >/dev/null 2>&1; do
    echo "Waiting for Docker to start..."
    sleep 5
done
echo "Docker is UP! Building and starting cluster..."
docker-compose up -d --force-recreate
sleep 15
docker logs thunder_bootnode | tail -n 20
curl -s -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"thunder_getBlockHeight","params":[],"id":1}' http://127.0.0.1:8080
