# Lotse sync-worker

Cloudflare Worker implementing `docs/SYNC_PROTOCOL.md`. The service is
intentionally dumb: it stores ciphertext and metadata (account IDs, device
names, HLC timestamps, sizes) and never sees plaintext -- see
`docs/THREAT_MODEL.md` section 3 for the exact list of what it stores (and
what it never stores).

No runtime dependencies. Password/session hashing uses WebCrypto (PBKDF2-
SHA256, 600 000 iterations) per `SYNC_PROTOCOL.md` section 8, since the
Worker's 10 ms CPU budget per request rules out running Argon2id server-side
-- the client-side Argon2id stretch (see `THREAT_MODEL.md`) is what actually
carries the security. Routing is a small hand-rolled router
(`src/router.ts`); no hono, no itty-router.

## Layout

```
src/
  index.ts    entry point / route table
  router.ts   tiny path-param router + auth middleware wiring
  auth.ts     /v1/auth/*, /v1/devices
  sync.ts     /v1/sync/push|pull|status
  blobs.ts    /v1/blobs/:id
  db.ts       all D1 SQL
  crypto.ts   WebCrypto helpers (PBKDF2, SHA-256, base64, ULID)
  errors.ts   ApiError + {error, message} JSON responses
  types.ts    Env, envelope, and row shapes shared across modules
migrations/0001_init.sql   exact schema from SYNC_PROTOCOL.md section 6
test/                      vitest + @cloudflare/vitest-pool-workers
```

## One-time setup

Already done for the production account (2026-09-13) — kept here so the setup can
be repeated on a fresh account.

```bash
npm install

# D1 database. Paste the printed database_id into wrangler.toml.
npx wrangler d1 create lotse --location weur

# R2 bucket. The `--jurisdiction eu` flag is NOT optional: it guarantees objects are
# stored *and processed* inside the EU, and it cannot be changed after creation.
# A bucket created without it lands wherever Cloudflare picks — for us, ENAM.
# Jurisdictional buckets live in a separate namespace: `wrangler r2 bucket list`
# and the API will NOT show them unless the jurisdiction is given as well. A 404
# there does not mean the bucket is missing.
npx wrangler r2 bucket create lotse-blobs --jurisdiction eu
```

D1 has no jurisdiction feature; `--location weur` is a hint, not a guarantee. That
difference matters for the privacy policy and is recorded here on purpose.

## Migrations

```bash
npm run migrate:local   # against the local Miniflare D1 (wrangler dev)
npm run migrate:remote  # against the real D1 database, after `d1 create`
```

## Run locally

```bash
npm run dev
```

Wrangler serves the worker with local D1 + R2 emulation. Exercise it with
`curl`, e.g.:

```bash
curl -s localhost:8787/v1/auth/prelogin?email=nobody@example.com
```

## Typecheck & test

```bash
npm run typecheck
npm test
```

Tests run inside the real Workers runtime (workerd) via
`@cloudflare/vitest-pool-workers`, against an in-memory D1 database that has
`migrations/0001_init.sql` applied fresh for every test file (see
`vitest.config.ts` and `test/apply-migrations.ts`). No external services or
network access are required to run `npm test`.

## Deploy

```bash
npm run migrate:remote   # once, and after every new migration
npm run deploy
```

The deploy also creates the Custom Domain `lotse-sync.sanctora.eu` from the `[[routes]]`
entry in `wrangler.toml`, including its DNS record. Do **not** create a DNS record for
that name by hand first — Cloudflare refuses a Custom Domain on a hostname that already
has a CNAME.

### No CORS headers — deliberate today, a decision for the browser client

The worker sends no `Access-Control-Allow-Origin` header, so a browser will refuse to
read its responses from any other origin. That is correct right now: the desktop app and
the CLI speak plain HTTP and are not subject to CORS.

It is, however, a hard blocker for the planned browser client, and the fix should be a
decision rather than a reflex. Two options, in order of preference:

1. **Serve the client from this worker** (Workers Static Assets). Same origin, so no CORS,
   no preflight round-trip, and the rate limiting rule covers the app as well as the API.
2. **Allow exactly one origin.** Workable, but it adds a preflight to every request and a
   list that has to be kept correct.

Do not reach for `Access-Control-Allow-Origin: *`. The endpoints are token-authenticated,
so it would not immediately leak data, but it removes a barrier for no gain.

### Rate limiting — required, and not in this code

The auth endpoints are deliberately **not** rate-limited in the worker: every request
that reaches the code has already cost CPU, and PBKDF2 with 600k iterations is exactly
what an attacker would want to trigger. The limit belongs in front of the worker.

Cloudflare dashboard → **Security** → **WAF** → **Rate limiting rules**:

| Field | Value |
|---|---|
| Path | `/v1/auth/*` |
| Requests | 10 |
| Period | 1 minute |
| Counting characteristic | IP address |
| Action | Block |

Without it, the free plan's CPU budget can be exhausted by anyone.

## Free-plan limits (SYNC_PROTOCOL.md section 8)

| Limit | Value | Design consequence |
|---|---|---|
| D1 writes/day | 100,000 rows | Client batches (2 s debounce); one push = one `db.batch()` transaction. |
| D1 reads/day | 5,000,000 rows | Pull uses the `records_seq` index, never a full scan. |
| Worker requests/day | 100,000 | Idle sync interval is 5 min ≈ 300 requests/day/device. |
| Worker CPU/request | 10 ms | No Argon2id in the Worker; server hashes `auth_key`/`recovery_auth_key` with PBKDF2-SHA256 (600k iterations) via WebCrypto instead. |
| R2 storage | 10 GB | Attachments over 25 MB are referenced by clients, not auto-uploaded. |

These limits are also documented as a comment directly in `wrangler.toml`.

**Auth-endpoint rate limiting is out of scope for this Worker.** Per-IP
throttling of `/v1/auth/*` (register/login/recover) should be configured as
a Cloudflare WAF rate limiting rule in front of the Worker -- not in Worker
code -- so abusive requests are stopped before they spend CPU on PBKDF2. See
the comment in `wrangler.toml` and in `src/auth.ts`.

## Known gaps / things a production deployment should revisit

- **`server_seq` assignment under concurrency.** `push` reads the current
  `seq` value, computes new sequence numbers in JS, then commits everything
  in one `db.batch()`. Two concurrent pushes for the *same account* racing
  between that read and their batch commit could both compute overlapping
  `server_seq` values. Given the product's single-owner-multi-device design
  and 2 s client-side debounce, this is unlikely in practice, but it is not
  serialized at the database level. A future iteration could move the
  counter increment into a `... RETURNING` statement inside the same batch,
  or use a Durable Object per account as the sync entry point.
- **Email enumeration on `/v1/auth/prelogin`.** It returns 404 for an
  unknown email, which a determined attacker could use to test whether an
  address has an account. `SYNC_PROTOCOL.md` doesn't specify a mitigation
  here (`auth_key` cannot be computed without the salt, so login itself
  can't be timing-attacked this way); flagged here rather than silently
  guessed at.
- **Blob size enforcement trusts `Content-Length`.** The Worker rejects
  uploads whose declared `Content-Length` exceeds 100 MiB, but does not
  independently re-count bytes streamed into R2. R2's own object size is
  recorded in the `blobs` table after the `put()` resolves.
