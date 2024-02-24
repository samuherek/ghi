-- This file should undo anything in `up.sql`
DELETE FROM lessons WHERE (id = 1 AND name = 'default');
