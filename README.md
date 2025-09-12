# Url Shortener
This is a self-hostable url shortener that can be setup with our own domain.

I build the base of it using this awesome video tutorial:
https://youtu.be/9KkTd4eDUMY
by https://github.com/oliverjumpertz

## Features

### For Users
- generated url aliases like https://example.com/MTE2NzQzMzM
- custom url aliases allowing sub paths like https://example.com/blog/my-article

### For Admins
- Prometheus integration
- Open Telemetry
- Dockerfile
- SQL Migrations for simple CI/CD

## Local Development

### Setup

You need to create a .env file in the root folder of the project and define these properties:
```dotenv
DATABASE_URL=
POSTGRES_USERNAME=
POSTGRES_PASSWORD=
```

### Starting
To start the postgres db locally:
```shell
sudo docker compose -f docker-compose-local-db.yml up
```

Or without docker compose (make sure to replace the username and password with your own from the .env file):
```shell
sudo docker run --name postgres -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres
```

Also, useful:
`docker compose build` to rebuild the image
`docker compose up` to start the api and db

### Migrations
To apply migrations:
```shell
sqlx migrate run
```

### Using the url-shortener.http file

Setup http-client.env.json files in the following structure:
```json
{
  "local": {
    "url": "",
    "api-key": ""
  },
  "prod": {
    "url": "",
    "api-key": ""
  }
}
```