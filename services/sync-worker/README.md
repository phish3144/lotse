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

```bash
npm install

# Create the D1 database, then paste the printed database_id into
# wrangler.toml (it currently has a placeholder UUID).
npx wrangler d1 create lotse

# Create the R2 bucket (name must match wrangler.toml's `bucket_name`).
npx wrangler r2 bucket create lotse-blobs
```

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
npm run deploy
```

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
