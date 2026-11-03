> **Parked branch: do not merge.** This branch (`docs/sync-spec`) holds a possible future
> implementation, kept for later. `main` should not follow it: nothing here is the plan of record,
> and `docs/spec.md` on `main` doesn't mention sync.

# Sync: what the remote server accepts and returns

*(Draft, 2026-11-03. Not built: no engine, shell or server code exists yet.)*

Sync keeps what the predictions learned the same on every device a person types on. The server is
one the user runs, reached over HTTPS: the owner's is a module on public.sjun.me, specified in the
coordination server spec (kept outside the repo). JIS Moonshot ships no server and no default URL. Sync is planned for JIS Korean too, so the names here have a `<Product>` placeholder:
`JIS Moonshot` / `JISMoonshot` / `JIS_MOONSHOT` here, `JIS Korean` / `JISKorean` / `JIS_KOREAN`
there.

**For advanced users only.** There's no switch in any menu or settings window. Sync is on only
where someone wrote a config file (or pointed an environment variable at one) by hand. A typist
who never did that can't turn it on by accident, and nothing they type can leave the device
(spec §10.11).

## 1. Principles

- **Decoding stays on the device.** The server never sees a key press or a reading, and typing
  never waits for it. With the server down, everything works as it does without sync.
- **The server is a dumb store.** It keeps opaque, encrypted records, one per device and kind, and
  returns them. It can't read them and never merges them. Merging is the engine's job (§6), pure and
  deterministic like the rest of it. Shells only move bytes.
- **End-to-end encrypted, always.** Records are sealed on the device with a key the server never
  gets (§5). There's no unencrypted mode.
- **No text, no keys, no times of typing, no app names** in what's synced today, as in what the
  learner stores (spec §10.11). The one kind that would carry text is reserved, and off unless named
  (§4).
- **Each device writes only its own records.** Two devices never write the same record, so the
  server only has to guard a device against itself (two processes, a restored backup).

## 2. Configuration

### 2.1 The config file

JSON, in the folder that holds the training file (spec §10.12):

- **macOS:** `~/Library/Application Support/<Product>/sync.json`
- **Windows:** `%APPDATA%\<Product>\sync.json`

```json
{
  "format": "jis-sync-config",
  "version": 1,
  "server": "https://sync.example.com",
  "token": "jsr_… (this device's refresh token, §3.1)",
  "key": "base64 of 32 random bytes, the same on every device",
  "kinds": ["learner", "weights", "training"],
  "interval": 900
}
```

| Field | Required | Meaning |
|---|---|---|
| `format`, `version` | yes | `"jis-sync-config"`, `1` |
| `server` | yes | Base URL of the server, without `/v1` |
| `token` | yes | This device's refresh token (§3.1). Written by `<Product> sync login` and on every rotation |
| `key` | yes | The encryption key (§5): 32 bytes, base64 (standard alphabet, padded) |
| `kinds` | no | What to sync (§4). Default `["learner", "weights"]`. Unknown kinds are ignored |
| `interval` | no | Seconds between syncs, 300–86400. Default 900 (15 minutes) |
| `device` | no | Set by the shell on first run, never by hand: this device's id (§3.2) |

**The file is refused, and sync stays off, when:**
- it's readable by anyone but the user (macOS: any group or other permission bit, so it must be
  `0600`; Windows: an ACL granting anyone but the user, SYSTEM and Administrators);
- `server` isn't `https://` (plain `http://` only to a loopback address, for testing);
- any required field is missing or malformed, or `key` isn't 32 bytes.

A refusal is logged once per change of the file (the reason, never the token or key) and shown in
the status line (§2.3).

The shell writes the file back for two things only: to add `device` on the first run, and to
store `token` (`<Product> sync login`, and each rotation, §3.1). It keeps the file's permissions.

### 2.2 Environment variables

The input method is started by the system, not a shell, so environment variables are a secondary
route (macOS: `launchctl setenv`, then restart the input method; Windows: user environment
variables, then sign out and back in).

| Variable | Meaning |
|---|---|
| `<PRODUCT>_SYNC_CONFIG` | Path to the config file instead of the default (same permission checks) |
| `<PRODUCT>_SYNC` | `0` turns sync off even with a config file. Nothing else turns it on |

There's no variable for the token or key: they live only in a file only the user can read.

### 2.3 What the user sees

There's no setting, but sync is never invisible. While a config file exists, the read-only "Learned
Weights" panel (spec §10.11) shows one line: *Synced with sync.example.com, 12:03* or *Sync:
config refused (file readable by others)*. That way someone who didn't set sync up can tell it's
on. Turning it off means deleting the file. **[decide]** whether this line stays (the stated
default) or sync shows nothing at all.

### 2.4 Who syncs

One process per user session talks to the server:

- **macOS:** the input method's process (there's one).
- **Windows:** the TIP runs inside every app's process and must never touch the network, so a small
  helper (`<Product>Sync.exe`, started at sign-in like the macOS login agent) does it. It reads and
  writes the same `HKCU` values the TIPs already share (`Learned`, `Weight.*`). **[decide]** the
  helper's name and how it starts.
- **Web playground:** never. It keeps the 24-hour rule.

## 3. The server API

HTTP/1.1 or later, JSON bodies (`Content-Type: application/json`, UTF-8), every path under `/v1`.
Base64 fields use the standard alphabet with padding.

### 3.1 Accounts and tokens

Tokens belong to one **device** of one **account** (a person's devices). They're issued after a
passkey sign-in (a security key such as a YubiKey, or a synced passkey such as Google Password
Manager's), started by `<Product> sync login`: a command, not a menu item, that opens the sign-in
page in the browser or prints a QR code for a phone, then writes the refresh token into the config
file. The sign-in flows are the coordination server spec's (kept outside the repo).

- **Refresh token** (`jsr_…`, in the config file): exchanged at `POST /v1/auth/token` for an
  access token and a **new** refresh token. The client stores the new one before using the access
  token. A refresh token used again after a short grace is treated as leaked, and the server
  revokes the device.
- **Access token** (`jsa_…`, in memory only): sent as `Authorization: Bearer <token>` on every
  `/v1/records` call. It lasts minutes; the client refreshes it when it's about to expire, or once
  after a 401.
- A token only writes its own device's records (§3.4). The server only ever returns the caller's
  own account's records. Revoking a device is done on the server's account page, not by the
  client.

### 3.2 Records

A record is one kind (§4) from one device. The device id is a random UUID (version 4), made by the
shell on first run and stored in the config file. It's never derived from the hostname or hardware.

```json
{
  "device": "6f1c1e9e-6a64-4f0f-9a39-8d2b1b6a7c11",
  "kind": "learner",
  "generation": 42,
  "nonce": "base64 of 24 bytes",
  "data": "base64 of the sealed payload (§5)",
  "stored": 1793664180000,
  "cursor": "opaque"
}
```

| Field | Set by | Meaning |
|---|---|---|
| `device` | client | The writing device's id |
| `kind` | client | `[a-z][a-z0-9-]{0,31}` |
| `generation` | client | 1 for the first write, then +1 each write. The server checks it (§3.4) |
| `nonce`, `data` | client | The sealed payload. The server stores them as given |
| `stored` | server | When the server stored it, Unix milliseconds |
| `cursor` | server | Where this write sits in the account's change order (§3.5) |

The server must not read, decode or change `nonce` and `data`. Its only checks are on their size and
base64 encoding.

### 3.3 `GET /v1`

What the server supports. It needs no token, so a client can check the URL before sending one.

```json
{ "protocol": [1], "maxRecordBytes": 1048576, "minInterval": 300 }
```

- `protocol`: the versions it speaks. A client that shares none stops and shows *Sync: server
  doesn't speak protocol 1*.
- `maxRecordBytes`: the largest decoded `data` it accepts, at least 1 MiB.
- `minInterval`: seconds a client must wait between syncs. The client uses the larger of this and
  its own `interval`.

### 3.4 `PUT /v1/records/{device}/{kind}`

Writes this device's record of one kind. Body: `{ "generation", "nonce", "data" }`.

- `generation` must be exactly the stored record's generation + 1, or 1 if there's none.
- **200** `{ "generation": 42, "stored": …, "cursor": "…" }`: stored.
- **409** `{ "error": "generation", "generation": 41 }`: it isn't the next one. The client pulls
  (§3.5), merges its own stored record into its state (§6), and writes again with the next
  generation. This only happens when one device wrote from two places.
- **413**: `data` is over `maxRecordBytes`. The client logs it and skips that kind until it changes.
- **400** `{ "error": "malformed" }`: a bad path, kind, base64 or field.
- **403** `{ "error": "device" }`: `{device}` isn't the token's device.

A client writes a kind only when it has changed since its last write.

### 3.5 `GET /v1/records?since={cursor}`

Every record in the account changed after `cursor`. Without `since`, all of them.

```json
{
  "records": [ { "device": "…", "kind": "learner", "generation": 42, "nonce": "…", "data": "…", "stored": …, "cursor": "…" } ],
  "deleted": [ { "device": "…", "kind": null } ],
  "cursor": "opaque",
  "more": false
}
```

- `records`: the latest version of each changed record (not every generation), in change order.
- `deleted`: records removed since `cursor` (§3.6). `kind: null` means every record of that device.
- `cursor`: what to pass next time. It's opaque to the client and kept between runs.
- `more`: true when the server cut the page short. The client asks again with the new cursor right
  away.
- **410** `{ "error": "cursor" }`: the cursor is too old or unknown. The client pulls again without
  `since`.

The client's own records come back too, and it uses them to recover after a 409 or a reinstall.

### 3.6 Deleting

- `DELETE /v1/records/{device}`: every record of the token's own device ("Reset What It Learned"
  on it). **204**; **403** for another device.
- Deleting another device's records, or the whole account's, needs a passkey: it's done on the
  server's account page, never by a device token or by regular sync.

A server keeps a deletion in `deleted` for at least 30 days, so other devices hear of it, and may
answer older cursors with 410.

### 3.7 Errors and backoff

Every error body is `{ "error": "<code>", "message": "<for logs>" }`.

| Status | Client does |
|---|---|
| 401 | Refreshes the access token once and retries. If the refresh token is refused too: stops syncing until the config file changes. Status: *signed out, run `sync login`* |
| 429, 503 | Waits `Retry-After` seconds (default: the backoff below) |
| other 4xx | Logs it, skips that request, carries on next interval |
| 5xx, network error | Exponential backoff: 1 min, doubling, at most 6 hours, ±10% jitter |

Requests time out after 30 seconds. A failed sync changes nothing on the device.

## 4. Kinds

What's inside `data` once it's opened (§5): a JSON object `{ "kind", "schema", "updated", "payload" }`.
`schema` is the kind's payload version and `updated` is the device's clock in Unix milliseconds
when the payload last changed.

| Kind | Default | Payload | Holds text? |
|---|---|---|---|
| `learner` | on | This device's own learner counts: `jk_learner_export`'s line (`jkl1 …`) | no |
| `weights` | on | The sliders set by hand (spec §10.5): `{ "<Scenario>": { "value": 1.2, "updated": … } }`, with `"value": null` for one cleared by hand | no |
| `training` | off | The training file as applied (spec §10.12): the JSON object, plus `updated`. Vocabulary and avoid lists are words | **yes: words** |
| `textmodel` | off | **Reserved** for the personal text model of the planned decoder (word counts from what was committed). Not specified until the decoder is | **yes** |

A kind listed in `kinds` that this build doesn't know is ignored, never sent.

Per-app settings (spec §10.11.1) aren't synced: they name apps, and apps differ between machines.

## 5. Sealing

- **Cipher:** XChaCha20-Poly1305 with the config's 32-byte `key` and a random 24-byte `nonce` per
  write.
- **Associated data:** the UTF-8 bytes of `jis-sync/1|{device}|{kind}|{generation}`. A record
  replayed under another device, kind or generation fails to open.
- **Plaintext:** the JSON object of §4.
- **The key** is made once (for example `openssl rand -base64 32`) and copied by hand to each
  device's config file. A lost key means the records can't be read: delete them (§3.6) and start
  again with a new one.
- A record that fails to open is skipped and logged (*a record from device … didn't open: another
  key?*). It never stops the others.

Sealing and opening are pure functions in the engine, so both shells share one implementation.
**[decide]** which crate (for example `chacha20poly1305`) and whether the engine takes on the
dependency or a shared sync crate beside it does.

**What the server still learns:** how many devices an account has, each record's size, and when
each device syncs (with the default interval this shows roughly when it's in use, not what's
typed).

## 6. Merging (engine)

The engine keeps **its own** state and **the others'** apart, so a device never re-uploads what it
received.

- **`learner`:** the counts this device records are its own and are what it uploads. The other
  devices' latest counts are held beside them, and the weights the predictor uses come from the
  sum: own + each other device's (each table summed, `events` not). Each device halves only its
  own counts. A device deleted (§3.6) drops out of the sum.
- **`weights`:** per scenario, the entry with the latest `updated` wins, ties broken by the larger
  device id. A `null` value clears the hand-set weight on every device. The learned weight then
  applies again.
- **`training`:** the payload with the latest `updated` is applied, as if that file were on this
  device. A local file changed later than it uploads and wins. **[decide]** whether a synced
  training file is written to the local folder or only applied.

New engine functions, all pure (names not settled): seal and open a record; export a kind's
payload; take every other device's payloads for a kind and apply the merged result. The shell
calls them in order: pull, open, merge, export what changed, seal, push.

## 7. A sync, step by step

At start (once the config is accepted), then every `interval` seconds (±10%), and when the device
wakes from sleep:

1. `GET /v1` the first time, and again after any error.
2. `GET /v1/records?since=<cursor>` until `more` is false, then store the cursor.
3. Open each record (§5) and hand the opened payloads to the engine (§6). The devices' engines pick
   the merged state up as they pick up `Learned` today.
4. For each kind in `kinds` that changed locally since its last push: seal it and
   `PUT /v1/records/<device>/<kind>`. On 409 go back to step 2 once, then try at the next interval.

Nothing in a sync runs on the key path.

## 8. Testing (when it's built)

- Engine: seal and open round trips, a wrong key, a record replayed under another device, kind or
  generation, and a merge whose result doesn't depend on the order records arrive in.
- Learner: three simulated devices learning one habit reach the same weights as one device that
  made all their choices (within the halving's difference). A deleted device's counts leave the
  sum.
- Config: each refusal of §2.1 (open permissions, plain `http` outside loopback, a
  short key), and `<PRODUCT>_SYNC=0`.
- A reference server small enough to run in the tests, against this document's status codes.

## 9. Open questions

- **[decide]** The status line of §2.3: shown (default) or nothing.
- **[decide]** The Windows helper (§2.4).
- **[decide]** The crypto dependency's home (§5).
- **[decide]** Whether a synced training file is written locally (§6).
- `textmodel`'s payload, once the decoder exists.
