-- Add down migration script here

-- Drop tables in dependency-safe order (transactions depends on accounts/categories)
DROP TABLE IF EXISTS transactions;
DROP TABLE IF EXISTS categories;
DROP TABLE IF EXISTS accounts;

-- Drop ENUM types (only if no tables still use them)
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'account_type') THEN
    DROP TYPE account_type;
  END IF;

  IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'transaction_type') THEN
    DROP TYPE transaction_type;
  END IF;

  IF EXISTS (SELECT 1 FROM pg_type WHERE typname = 'category_type') THEN
    DROP TYPE category_type;
  END IF;
END$$;

DROP EXTENSION IF EXISTS "uuid-ossp";
