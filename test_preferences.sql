-- Test SQL for preferences table
-- This file tests inserting and querying preferences from the database

-- Insert test preferences
INSERT INTO preferences (user_id, display_name, color_scheme, animations, animation_type, quote_source, greeting_style, updated_at)
VALUES (1, 'Moses', 'vibrant', 1, 'matrix', 'all', 'dynamic', strftime('%s', 'now'));

INSERT INTO preferences (user_id, display_name, color_scheme, animations, animation_type, quote_source, greeting_style, updated_at)
VALUES (2, 'TestUser', 'pastel', 0, 'starfield', 'coding', 'formal', strftime('%s', 'now'));

-- Query all preferences
SELECT * FROM preferences;

-- Query specific user preferences
SELECT display_name, color_scheme, animations, animation_type, quote_source, greeting_style 
FROM preferences 
WHERE user_id = 1;

-- Update preferences
UPDATE preferences 
SET color_scheme = 'minimal', animations = 0, updated_at = strftime('%s', 'now')
WHERE user_id = 1;

-- Verify update
SELECT * FROM preferences WHERE user_id = 1;

-- Test INSERT OR REPLACE (upsert)
INSERT OR REPLACE INTO preferences (user_id, display_name, color_scheme, animations, animation_type, quote_source, greeting_style, updated_at)
VALUES (1, 'Moses Updated', 'vibrant', 1, 'particles', 'all', 'casual', strftime('%s', 'now'));

-- Verify upsert
SELECT * FROM preferences WHERE user_id = 1;

-- Delete test data
DELETE FROM preferences WHERE user_id IN (1, 2);

-- Verify deletion
SELECT COUNT(*) as preference_count FROM preferences;
