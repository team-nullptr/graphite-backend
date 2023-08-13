Backend server for Graphite.

1. Run `docker compose up`
2. Run server with `cargo run`

Connecting with locally created database

```
mysql -u root -h 127.0.0.1 -P 3306 -p
```

In order to run the app you will need `/config/dev.toml` file. Here is the template:

```
[database]
connection = "postgres://postgres:dev@localhost:5432/database"

[server]
port = 3000
address = "127.0.0.1"

[oauth]
github_client_id="<github_client_id>"
github_secret_id="<github_secret_id>"
```
