-- Migrate items from quantity-based to durability-based wear model.
-- All existing items are wiped (no backfill); items will be re-spawned
-- by the game engine using the new durability ranges per rarity.
DELETE owns;
DELETE items;
DELETE item;
