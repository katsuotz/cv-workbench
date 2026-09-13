# CV Workbench production deployment

The production stack runs PostgreSQL 19 Beta 3, the Rust API, and the XeLaTeX worker with Docker Compose. The API is published on loopback port `18732`; PostgreSQL has no host port. The service memory limits leave room for the existing applications on the server. The existing host Nginx can proxy the API and serve the frontend after the domain is configured.

Build the frontend with `PUBLIC_SITE_URL=https://cvworkbench.com` so canonical URLs, social
previews, structured data, and the sitemap point to the public site. The frontend defaults to this
domain when the variable is not set; use the local origin from `frontend/.env.example` for local
development.

When serving the static frontend, route client-side paths such as `/app` to the generated
`200.html` fallback while serving the prerendered `/index.html` directly.

## Required files

Copy `.env.production.example` to `.env` on the server and replace the placeholder database password. Keep the file readable only by the deployment user or root.

Set `RELEASE_TAG` to the release being deployed. The API and worker images must use the same tag.

## Start the stack

Load the API and worker images built for the release before starting the stack:

```sh
docker load --input cvworkbench-api-${RELEASE_TAG}.tar
docker load --input cvworkbench-worker-${RELEASE_TAG}.tar
docker compose -f compose.production.yaml config
docker compose -f compose.production.yaml up -d --no-build
```

The API runs SQLx migrations on startup. Check both health endpoints after the API becomes healthy:

```sh
curl --fail http://127.0.0.1:18732/health/live
curl --fail http://127.0.0.1:18732/health/ready
```

The `COOKIE_SECURE` setting remains `false` until HTTPS is active. Change it to `true` and recreate the API and worker containers after TLS is configured.
