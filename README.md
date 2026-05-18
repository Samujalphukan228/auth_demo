# auth_demo

Authentication demo built using [ezconfig-rs](https://crates.io/crates/ezconfig-rs)

> A production-grade reusable auth starter built in Rust.

---

## Stack

| Tool | Purpose |
|------|---------|
| Axum 0.8 | Web framework |
| MariaDB + sqlx | Permanent storage |
| Redis + deadpool-redis | Token storage, rate limiting, blocklist |
| argon2 | Password hashing |
| jsonwebtoken | JWT access + refresh tokens |
| ezconfig-rs | Zero-boilerplate config loading |
| Brevo API | Transactional email (verification, reset) |
| reqwest | HTTP client for Brevo |

---

## Features

- Email verification via click link (UUID token stored in Redis, 15 min TTL)
- Auto-resend verification email if user registers again before verifying
- Atomic registration — rolls back user creation if email sending fails
- HttpOnly cookie-based auth (no tokens in localStorage or headers)
- Short-lived access tokens (15 min) + long-lived refresh tokens (30 days)
- JWT blocklist on logout (Redis, auto-expires)
- Rate limiting on login — 5 attempts per IP per 15 min (Redis)
- Session management — see all active devices, logout one or all
- Password confirmation required for session management
- Forgot password via email link (UUID token, 15 min TTL)
- Password reset force-logs out all devices
- Repository pattern — all SQL in one place, never in handlers
- Service layer — all business logic isolated, no HTTP concerns
- Global error handling with automatic HTTP response conversion
- No `unwrap` or `expect` in production code

---

## Project Structure

```
auth_demo/
├── Cargo.toml
├── .env
└── src/
    ├── main.rs                        → server bootstrap only
    ├── app.rs                         → setup: config, DB, Redis, state
    ├── config.rs                      → typed config via ezconfig-rs
    ├── state.rs                       → AppState (db + redis + config)
    ├── errors.rs                      → global AppError with IntoResponse
    ├── db/
    │   └── mod.rs                     → MariaDB connection pool
    ├── cache/
    │   └── mod.rs                     → Redis connection pool
    ├── middleware/
    │   └── auth_guard.rs             → JWT extractor from HttpOnly cookie
    ├── utils/
    │   ├── jwt.rs                     → JWT encode/decode
    │   ├── hash.rs                    → argon2 hash + verify
    │   └── email.rs                   → Brevo API email sending
    ├── routes/
    │   └── mod.rs                     → all routes registered here
    └── features/
        └── auth/
            ├── mod.rs
            ├── model.rs               → User struct (full DB model)
            ├── dto.rs                 → request/response structs
            ├── repository.rs          → all SQL queries
            ├── service.rs             → business logic
            └── handler.rs             → route handlers
```

---

## Database Schema

```sql
CREATE TABLE users (
    id VARCHAR(36) PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE refresh_tokens (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    device VARCHAR(255),
    ip VARCHAR(45),
    expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

---

## Environment Variables

```env
DATABASE_URL=mysql://root:password@localhost/auth_demo
REDIS_URL=redis://127.0.0.1:6379
JWT_SECRET=your_long_random_secret
JWT_REFRESH_SECRET=your_long_random_refresh_secret
BREVO_API_KEY=your_brevo_api_key
BREVO_SENDER_EMAIL=noreply@yourdomain.com
BREVO_SENDER_NAME=Your App
PORT=8080
APP_URL=http://localhost:8080
```

---

## API Routes

### Auth

| Method | Route | Auth | Description |
|--------|-------|------|-------------|
| POST | `/auth/register` | No | Register new user, send verification email |
| GET | `/auth/verify?token=` | No | Verify email via link |
| POST | `/auth/login` | No | Login, sets HttpOnly cookies |
| POST | `/auth/refresh` | Cookie | Get new access token |
| POST | `/auth/logout` | Cookie | Logout current device |
| POST | `/auth/forgot-password` | No | Send password reset email |
| GET | `/auth/reset-password?token=` | No | Validate reset token |
| POST | `/auth/reset-password` | No | Set new password |

### Sessions

| Method | Route | Auth | Description |
|--------|-------|------|-------------|
| GET | `/auth/sessions` | Cookie | List all active devices |
| DELETE | `/auth/sessions/:id` | Cookie | Logout specific device (password required) |
| DELETE | `/auth/sessions` | Cookie | Logout all devices (password required) |

### User

| Method | Route | Auth | Description |
|--------|-------|------|-------------|
| GET | `/me` | Cookie | Get current user profile |

---

## Request / Response Examples

### Register
```http
POST /auth/register
Content-Type: application/json

{
  "email": "sam@example.com",
  "password": "password123"
}
```
```json
{ "message": "Registration successful. Please check your email to verify your account." }
```

### Login
```http
POST /auth/login
Content-Type: application/json

{
  "email": "sam@example.com",
  "password": "password123"
}
```
```json
{ "message": "Logged in successfully." }
```
Cookies set automatically:
```
Set-Cookie: access_token=eyJ...; HttpOnly; SameSite=Strict; Max-Age=900; Path=/
Set-Cookie: refresh_token=eyJ...; HttpOnly; SameSite=Strict; Max-Age=2592000; Path=/
```

### Get Profile
```http
GET /me
```
```json
{
  "id": "uuid",
  "email": "sam@example.com",
  "is_verified": true
}
```

### Get Sessions
```http
GET /auth/sessions
```
```json
[
  {
    "id": "uuid",
    "device": "Mozilla/5.0 Chrome/...",
    "ip": "192.168.1.1",
    "created_at": "2026-05-18T22:00:00",
    "is_current": true
  }
]
```

### Forgot Password
```http
POST /auth/forgot-password
Content-Type: application/json

{
  "email": "sam@example.com"
}
```
```json
{ "message": "If that email exists you will receive a reset link shortly." }
```

### Reset Password
```http
POST /auth/reset-password
Content-Type: application/json

{
  "token": "uuid-from-email",
  "new_password": "newpassword123"
}
```
```json
{ "message": "Password reset successfully. Please log in again." }
```

### Logout Specific Session
```http
DELETE /auth/sessions/:session_id
Content-Type: application/json

{
  "password": "password123"
}
```
```json
{ "message": "Session logged out successfully." }
```

### Logout All Sessions
```http
DELETE /auth/sessions
Content-Type: application/json

{
  "password": "password123"
}
```
```json
{ "message": "All sessions logged out successfully." }
```

---

## Redis Key Patterns

| Key | Value | TTL | Purpose |
|-----|-------|-----|---------|
| `verify:TOKEN` | user_id | 15 min | Email verification token |
| `reset:TOKEN` | user_id | 15 min | Password reset token |
| `login_attempts:IP` | count | 15 min | Rate limiting |
| `blocklist:TOKEN` | "1" | 15 min | Revoked access tokens |

---

## Security

- Passwords hashed with argon2id
- JWTs signed with HS256
- Tokens stored in HttpOnly cookies — immune to XSS
- SQL injection impossible — all queries use parameterized statements via sqlx
- Rate limiting on login prevents brute force
- Token blocklist prevents use of stolen access tokens after logout
- Generic error messages — never reveals if email exists on forgot password
- Email must be verified before login is allowed
- Password reset invalidates all active sessions across all devices

---

## Running Locally

```bash
# start dependencies
sudo systemctl start mariadb
sudo systemctl start redis

# create database
mysql -u root -p -e "CREATE DATABASE IF NOT EXISTS auth_demo;"

# run
cargo run
```

Server starts on `http://0.0.0.0:8080`

---

## Adding a New Feature

To add a new feature (e.g. notes, todos, bookings) follow the same pattern:

```
src/features/
└── notes/
    ├── mod.rs
    ├── model.rs
    ├── dto.rs
    ├── repository.rs
    ├── service.rs
    └── handler.rs
```

Register routes in `src/routes/mod.rs`. Zero changes to existing files.

---

## License

MIT — built by [Samujal Phukan](https://github.com/Samujalphukan228) / NexxUpp
