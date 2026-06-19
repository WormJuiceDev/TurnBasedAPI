# FriendBox

FriendBox is an API-only Rust backend for the Android FriendBox client.

The backend stores identity, sessions, game records, opaque game state, and
opaque turn payloads. It does not make game decisions, validate game rules,
calculate winners, score games, or interpret gameplay JSON.

## Live API

```text
https://friendbox.youworld.app
```

Local development defaults to:

```text
http://127.0.0.1:3000
```

## Authentication

Register and login return a bearer token. Store it on the Android client and
send it on authenticated requests:

```text
Authorization: Bearer YOUR_TOKEN
```

Logout revokes only the current token. Other devices or sessions stay logged in.

Error responses use this shape:

```json
{
  "status": "error",
  "error": "invalid_or_missing_token"
}
```

## Endpoints

```text
GET  /health
GET  /db-health

POST /api/v1/auth/register
POST /api/v1/auth/login
POST /api/v1/auth/logout

GET   /api/v1/me
PATCH /api/v1/me
DELETE /api/v1/me

GET  /api/v1/players/search?username=player

GET  /api/v1/games
POST /api/v1/games
GET  /api/v1/games/{game_id}
POST /api/v1/games/{game_id}/start
POST /api/v1/games/{game_id}/turns

GET  /api/v1/game-invites
POST /api/v1/game-invites
POST /api/v1/game-invites/{invite_id}/accept
POST /api/v1/game-invites/{invite_id}/decline
```

## Auth Examples

Register:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"savvy","password":"TestPassword123","display_name":"Savvy"}'
```

Login:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"savvy","password":"TestPassword123"}'
```

Logout the current token:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/auth/logout \
  -H "Authorization: Bearer TOKEN"
```

Fetch current player:

```sh
curl https://friendbox.youworld.app/api/v1/me \
  -H "Authorization: Bearer TOKEN"
```

Update display name:

```sh
curl -X PATCH https://friendbox.youworld.app/api/v1/me \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer TOKEN" \
  -d '{"display_name":"Savvy"}'
```

Delete the current account:

```sh
curl -X DELETE https://friendbox.youworld.app/api/v1/me \
  -H "Authorization: Bearer TOKEN"
```

## Player Lookup

Search by username prefix:

```sh
curl "https://friendbox.youworld.app/api/v1/players/search?username=alex" \
  -H "Authorization: Bearer TOKEN"
```

The response contains public player records:

```json
[
  {
    "id": "PLAYER_ID",
    "username": "alex",
    "display_name": "Alex"
  }
]
```

## Game Invites

Create invites for a pending multiplayer game:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/game-invites \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer SAVVY_TOKEN" \
  -d '{"invitee_player_ids":["ALEX_PLAYER_ID","SAM_PLAYER_ID"],"client_game_type":"friendbox","initial_state":{"turn":0}}'
```

This creates one pending game owned by the current player and one pending
invite for each invited player. For simple two-player flows, the legacy
`invitee_player_id` field is still accepted.

List pending invites for the current player:

```sh
curl https://friendbox.youworld.app/api/v1/game-invites \
  -H "Authorization: Bearer ALEX_TOKEN"
```

Accept an invite:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/game-invites/INVITE_ID/accept \
  -H "Authorization: Bearer ALEX_TOKEN"
```

Decline an invite:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/game-invites/INVITE_ID/decline \
  -H "Authorization: Bearer ALEX_TOKEN"
```

Accepting an invite adds that player as a participant to the invite's pending
game. The host can start the game after at least one invited player has
accepted:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/games/GAME_ID/start \
  -H "Authorization: Bearer SAVVY_TOKEN"
```

The backend stores the final player count implicitly through the accepted
participants. Pending games reject turn submissions until the owner starts
them.

## Games And Turns

List games visible to the current player:

```sh
curl https://friendbox.youworld.app/api/v1/games \
  -H "Authorization: Bearer TOKEN"
```

Fetch one game, its participants, and its turn log:

```sh
curl https://friendbox.youworld.app/api/v1/games/GAME_ID \
  -H "Authorization: Bearer TOKEN"
```

Submit a turn:

```sh
curl -X POST https://friendbox.youworld.app/api/v1/games/GAME_ID/turns \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer TOKEN" \
  -d '{"client_turn_id":"android-local-id-1","turn_payload":{"move":"example"},"resulting_state":{"turn":1}}'
```

The backend stores `turn_payload` as opaque JSON. If `resulting_state` is
provided, it replaces the game's current opaque `state`. If it is omitted, the
turn is appended and the game's `updated_at` changes.

## Local Development

Run the server:

```sh
cargo run
```

Optional environment variables:

```text
FRIENDBOX_BIND_ADDR=0.0.0.0:3000
DATABASE_URL=postgres://friendbox_user:YOUR_REAL_PASSWORD@192.168.1.105:5432/friendbox_db
```

For deployment, create `/root/app/friendbox.env` from `friendbox.env.example`
and put the real database password there. Keep `friendbox.env` out of source
control and shared handoffs.

## Database Setup

Run these scripts in DBeaver while connected to `friendbox_db`:

```text
sql/001_initial_backend.sql
sql/002_username_password_auth.sql
sql/003_game_invites.sql
sql/004_multiplayer_lobbies.sql
```

No new migration is required for logout or display-name updates. The original
schema already includes `player_sessions.revoked_at` and `players.display_name`.

## Deployment

Copy the project files to the FriendBox server share, then run this inside the
`friendbox-app` jail:

```sh
cd /root/app
sh deploy.sh
```

The deploy script installs and enables the FreeBSD service:

```sh
service friendbox status
service friendbox restart
sysrc friendbox_enable
```

## Repo Layout

This workspace contains more than one Git context.

- The outer folder `D:\Studio\CodexFarm\FriendBox` is a separate workspace repo.
- The actual FriendBox Engine GitHub repo that has been pushed and should be
  checked first for engine work is:

```text
D:\Studio\CodexFarm\FriendBox\FriendBox Engine
```

For future engine sessions, run Git commands such as `git status` and
`git log` inside `FriendBox Engine`, not only from the outer `FriendBox`
folder, so the pushed engine repo state is clear from the start.

## Android Parity Workflow

For Android runtime and export parity work, Windows is the ground-truth
implementation.

Required approach for future sessions:

- At the start of each session, explicitly tell the user that the work will
  follow a Windows-first parity approach.
- Inspect the Windows implementation first before changing Android behavior.
- Mirror Windows behavior in Android as literally as practical instead of
  inventing a cleaner or simplified Android-specific version.
- Do not take shortcuts or re-design the feature when Windows already defines
  the intended behavior.
- If the user has already confirmed that an Android parity path works, treat
  that implementation as protected and do not replace, refactor, or reroute it
  while working on nearby features unless the Windows implementation itself
  requires the same coordinated change.
- If a nearby feature needs changes in shared rendering, physics, input, or VM
  flow, explicitly audit the already-confirmed Android parity paths that touch
  the same code before editing them, and preserve the known-good behavior.
- Only diverge from Windows behavior if there is a clear technical reason, and
  call that out explicitly before or when making the change.
- When investigating a mismatch, compare exact state updates, render math,
  collision rules, event behavior, and helper semantics between Windows and
  Android before patching.

This rule exists because the goal is full Android parity with the Windows VM
and runtime, not approximate feature similarity.
