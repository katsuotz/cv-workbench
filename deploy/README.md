# CV Workbench production deployment

The production stack runs PostgreSQL 19 Beta 3, the Rust API, the XeLaTeX worker, and the static frontend with Docker Compose. The API is published on loopback port `18732`, and the frontend on loopback port `18731`; PostgreSQL has no host port. The existing host Nginx terminates HTTPS and proxies the API, health checks, and frontend to the Compose services.

Build the frontend with `PUBLIC_SITE_URL=https://cvworkbench.com` so canonical URLs, social
previews, structured data, and the sitemap point to the public site. The frontend defaults to this
domain when the variable is not set; use the local origin from `frontend/.env.example` for local
development.

The frontend image uses Nginx to serve the generated static files and route client-side paths such
as `/app` to the generated `200.html` fallback.

## Required files

Copy `.env.production.example` to `.env` on the server and replace the placeholder database password. Keep the file readable only by the deployment user or root.

Set `RELEASE_TAG` to the release being deployed. The API, worker, and frontend images must use the same tag.

To enable Google sign-in, set `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, and
`GOOGLE_REDIRECT_URI` in the deployment environment. Register the exact callback URI in the
Google Cloud OAuth web application client. Keep the client secret readable only by the deployment
user or root; the frontend never receives it. For the current same-domain Nginx setup, use
`https://cvworkbench.com/api/v1/auth/google/callback` as the callback URI.

To enable LinkedIn sign-in, set `LINKEDIN_CLIENT_ID`, `LINKEDIN_CLIENT_SECRET`, and
`LINKEDIN_REDIRECT_URI`, using `https://cvworkbench.com/api/v1/auth/linkedin/callback` for the
same-domain Nginx setup. Full-history CV import also requires the approved LinkedIn import scopes,
`LINKEDIN_IMPORT_SCOPES`, and the corresponding `LINKEDIN_IMPORT_PROFILE_URL`. Keep the client secret
readable only by the deployment user or root;
the frontend never receives it.

## Start the stack

Log in to GHCR with a token that can read the packages, then pull the release images before
starting the stack:

```sh
printf '%s' "$GHCR_TOKEN" | docker login ghcr.io --username "$GHCR_USERNAME" --password-stdin
docker compose -f compose.production.yaml config
docker compose -f compose.production.yaml pull api worker frontend
docker compose -f compose.production.yaml up -d --no-build api worker frontend
docker logout ghcr.io
```

The API runs SQLx migrations on startup. Check both health endpoints after the API becomes healthy:

```sh
curl --fail http://127.0.0.1:18732/health/live
curl --fail http://127.0.0.1:18732/health/ready
```

The production deployment uses `COOKIE_SECURE=true` because HTTPS is active.

## GitHub Actions release deployment

Pushing a semantic version tag such as `v0.1.7` runs `.github/workflows/release.yml`. Frontend and backend validation, image builds, and GHCR pushes run in parallel. After both image sets are available, the workflow stages the shared Compose and Nginx configuration once, then deploys the frontend and the API/worker services in parallel. The backend rollout starts SQLx migrations through the API and waits for the API and worker; the frontend rollout waits for its HTTP health check before applying the Nginx configuration. The workflow verifies the public HTTPS endpoints after both rollouts.

Configure these repository secrets before pushing a release tag:

- `DEPLOY_HOST`: the server hostname or IP address
- `DEPLOY_USER`: the SSH user with access to `/opt/cvworkbench` and Docker
- `DEPLOY_SSH_PRIVATE_KEY`: the private key used by the workflow
- `DEPLOY_KNOWN_HOSTS`: the pinned `known_hosts` entry for the deployment server
- `GHCR_USERNAME`: the GitHub username used for the server registry login
- `GHCR_TOKEN`: a GitHub token with `read:packages` permission
