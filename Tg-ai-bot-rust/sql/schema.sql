
---

### sql/schema.sql

```sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS groups (
  id serial PRIMARY KEY,
  telegram_group_id bigint UNIQUE NOT NULL,
  name text
);

CREATE TABLE IF NOT EXISTS admins (
  id serial PRIMARY KEY,
  telegram_id bigint UNIQUE NOT NULL,
  name text,
  is_superadmin boolean DEFAULT false
);

CREATE TABLE IF NOT EXISTS authorized_users (
  id serial PRIMARY KEY,
  telegram_id bigint NOT NULL,
  group_id int REFERENCES groups(id)
);

CREATE TABLE IF NOT EXISTS api_keys (
  id serial PRIMARY KEY,
  provider text NOT NULL,
  key text NOT NULL,
  key_type text NOT NULL, -- search|llm|image
  quota bigint DEFAULT 0,
  priority int DEFAULT 100,
  status text DEFAULT 'active',
  last_used timestamptz
);

CREATE TABLE IF NOT EXISTS conversations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  chat_id bigint NOT NULL,
  user_id bigint NOT NULL,
  context jsonb,
  last_updated timestamptz DEFAULT now()
);