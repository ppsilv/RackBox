

sudo docker buildx build --platform linux/amd64 -t casaos-dashboard . --load --no-cache

sudo docker run -d -p 8080:8080 --name dashboard casaos-dashboard

curl http://localhost:8080/health

curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1001,"ip":"192.168.1.101","status":"inactive","port":26847,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1002,"ip":"192.168.1.202","status":"pending","port":15678,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1003,"ip":"192.168.1.041","status":"inactive","port":25647,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1004,"ip":"192.168.1.1d2","status":"pending","port":15648,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1005,"ip":"192.168.1.131","status":"inactive","port":25837,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1006,"ip":"192.168.1.082","status":"pending","port":15643,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1007,"ip":"192.168.1.171","status":"inactive","port":25837,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1008,"ip":"192.168.1.122","status":"pending","port":15643,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1009,"ip":"192.168.1.101","status":"inactive","port":26847,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1010,"ip":"192.168.1.202","status":"pending","port":15678,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1011,"ip":"192.168.1.041","status":"inactive","port":25647,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1012,"ip":"192.168.1.1d2","status":"pending","port":15648,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1013,"ip":"192.168.1.131","status":"inactive","port":25837,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1014,"ip":"192.168.1.082","status":"pending","port":15643,"cid":1002}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1015,"ip":"192.168.1.171","status":"inactive","port":25837,"cid":1001}}}' http://localhost:8080/data
curl -X POST -H "Content-Type: application/json" -d '{"Comando":{"client":{"id":1016,"ip":"192.168.1.122","status":"pending","port":15643,"cid":1002}}}' http://localhost:8080/data
