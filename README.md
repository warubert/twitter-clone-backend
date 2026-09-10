docker compose up -d
sqlx migrate run
docker exec -it twitter-clone-backend-postgres-1 psql -U postgres -d twitter
