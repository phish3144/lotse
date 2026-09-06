-- Lotse sync-worker schema, exact match of SYNC_PROTOCOL.md section 6.
CREATE TABLE accounts (
  id TEXT PRIMARY KEY, email TEXT UNIQUE NOT NULL,
  auth_hash TEXT NOT NULL, recovery_auth_hash TEXT NOT NULL,
  salt TEXT NOT NULL, kdf_m INTEGER NOT NULL, kdf_t INTEGER NOT NULL, kdf_p INTEGER NOT NULL,
  wrapped_account_key TEXT NOT NULL, wrapped_account_key_recovery TEXT NOT NULL,
  plan TEXT NOT NULL DEFAULT 'free', flags TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL
);
CREATE TABLE devices (
  id TEXT PRIMARY KEY, account_id TEXT NOT NULL REFERENCES accounts(id),
  name TEXT NOT NULL, platform TEXT NOT NULL,
  created_at INTEGER NOT NULL, last_seen_at INTEGER NOT NULL
);
CREATE TABLE sessions (
  token_hash TEXT PRIMARY KEY, account_id TEXT NOT NULL, device_id TEXT NOT NULL,
  created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL
);
CREATE TABLE records (
  account_id TEXT NOT NULL, id TEXT NOT NULL, kind TEXT NOT NULL,
  hlc TEXT NOT NULL, device_id TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0,
  format_version INTEGER NOT NULL, nonce TEXT NOT NULL, ciphertext TEXT NOT NULL,
  server_seq INTEGER NOT NULL,
  PRIMARY KEY (account_id, id)
);
CREATE INDEX records_seq ON records(account_id, server_seq);
CREATE TABLE seq (account_id TEXT PRIMARY KEY, value INTEGER NOT NULL);
CREATE TABLE blobs (
  account_id TEXT NOT NULL, id TEXT NOT NULL, size INTEGER NOT NULL,
  created_at INTEGER NOT NULL, PRIMARY KEY (account_id, id)
);
