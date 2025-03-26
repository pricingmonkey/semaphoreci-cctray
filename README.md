# semaphoreci-cctray

An adapter to publish the status of Jobs in SemaphoreCI in the cctray XML format, so that it can be consumed by build
monitors.

## How to run

Prerequisites: You need `docker` and `docker-compose`.

Open a terminal and move to the project directory. Then run:

```shell
docker compose up -d --build
```

This will run both `semaphoreci-cctray` and `nevergreen`. By default, you can then open:

- `sempahoreci-cctray` at http://localhost:8080
- `nevergreen` at http://localhost:5000

From nevergreen, you can connect to semaphoreci-cctray using `http://semaphoreci-cctray:8080`.

### Ports

You can configure the host ports using environment variables. Eg.

```shell
PORT_NEVERGREEN=5080 docker compose up -d --build
```

| Env Var                 | Description                                     | Default |
|-------------------------|-------------------------------------------------|---------|
| PORT_NEVERGREEN         | configures the host port for nevergreen         | 5000    |
| PORT_SEMAPHORECI_CCTRAY | configures the host port for semaphoreci-cctray | 8080    |
