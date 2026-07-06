-- Subunits: a container item (e.g. "1 packet") can track how many individual
-- pieces remain inside it (e.g. "3 eggs left").
--
--   unit               the container/outer unit, paired with `quantity`  → "1 packet"
--   subunit_remaining  how many individual pieces are left                → 3
--   subunit_unit       what those pieces are                              → "eggs"
--
-- All nullable: a plain item ("2 apples") simply leaves them unset.
ALTER TABLE items
    ADD COLUMN unit              TEXT,
    ADD COLUMN subunit_remaining INTEGER,
    ADD COLUMN subunit_unit      TEXT,
    ADD CONSTRAINT subunit_remaining_non_negative
        CHECK (subunit_remaining IS NULL OR subunit_remaining >= 0);
