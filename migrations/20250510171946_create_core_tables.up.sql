-- Add up migration script here

-- ENUM types
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'account_type') THEN
    CREATE TYPE account_type AS ENUM ('cash', 'bank', 'investment', 'other');
  END IF;

  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transaction_type') THEN
    CREATE TYPE transaction_type AS ENUM ('income', 'expense', 'transfer');
  END IF;

  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'category_type') THEN
    CREATE TYPE category_type AS ENUM ('income', 'expense', 'transfer', 'all');
  END IF;
END$$;

-- accounts table
CREATE TABLE accounts (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  type account_type NOT NULL,
  currency CHAR(3) NOT NULL DEFAULT 'TWD',
  initial_balance NUMERIC(14,2) NOT NULL DEFAULT 0.00,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- categories table
CREATE TABLE categories (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  type category_type NOT NULL,
  color CHAR(7),  -- ex: #FF0000
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- transactions table
CREATE TABLE transactions (
  id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
  account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  type transaction_type NOT NULL,
  amount NUMERIC(14,2) NOT NULL CHECK (amount > 0),
  category_id UUID REFERENCES categories(id),
  date DATE NOT NULL,
  note TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
