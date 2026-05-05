-- 1. Set default for all future rows
ALTER TABLE members ALTER COLUMN gender SET DEFAULT 'unspecified';

-- 2. Update existing NULLs to the new default
UPDATE members SET gender = 'unspecified' WHERE gender IS NULL;

-- 3. (Optional) Prevent future NULLs from being inserted manually
ALTER TABLE members ALTER COLUMN gender SET NOT NULL;
