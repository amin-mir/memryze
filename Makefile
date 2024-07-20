start-postgres:
	docker volume create memryze-pg
	docker run --name memryze-pg -e POSTGRES_PASSWORD=pswd -p 5432:5432 -d\
		--mount source=memryze-pg,target=/var/lib/postgresql/data\
		postgres:16-alpine

run-psql:
	docker run -it --rm --network host postgres:16-alpine psql -h localhost -p 5432 -U postgres

stop-postgres:
	docker kill memryze-pg
	docker rm memryze-pg

init-db:
	cargo run --bin db

run-memryze:
	cargo run --bin memryze

run-client:
	cargo run --bin client
