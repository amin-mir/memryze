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

run-srv-dev:
	RUST_LOG=info cargo run --bin memryze

build-srv-prd:
	# Make sure you set the env vars:
	# export ADDR="0.0.0.0:9871"
	# export POSTGRES_URI="postgres://memryze:oosata7Vou5roNge@localhost:5432/memryze"
	# export RUST_LOG="info"
	cargo build --release --bin memryze

run-srv-prd: build-srv-prd
	./target/release/memryze

run-client:
	cargo run --bin client


