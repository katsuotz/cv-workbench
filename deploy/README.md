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

To enable Google sign-in, set `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, and
`GOOGLE_REDIRECT_URI` in the deployment environment. Register the exact callback URI in the
Google Cloud OAuth web application client. Keep the client secret readable only by the deployment
user or root; the frontend never receives it. For the current same-domain Nginx setup, use
`https://cvworkbench.com/api/v1/auth/google/callback` as the callback URI.

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

The production deployment uses `COOKIE_SECURE=true` because HTTPS is active.

## GitHub Actions release deployment

Pushing a semantic version tag such as `v0.1.2` runs `.github/workflows/release.yml`. The workflow validates the Bun frontend and Rust backend, builds the production frontend and tagged Docker images, transfers them to the server, updates the `current` frontend symlink, recreates the API and worker, and verifies the public HTTPS endpoints.

Configure these repository secrets before pushing a release tag:

- `DEPLOY_HOST`: the server hostname or IP address
- `DEPLOY_USER`: the SSH user with access to `/opt/cvworkbench` and Docker
- `DEPLOY_SSH_PRIVATE_KEY`: the private key used by the workflow
- `DEPLOY_KNOWN_HOSTS`: the pinned `known_hosts` entry for the deployment server
