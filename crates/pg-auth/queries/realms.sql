--! refresh_realmlist
SELECT id, name, icon as realm_type, gamebuild as build, address as external_address, local_address, local_subnet_mask, port, flag as flags, timezone, population FROM realmlist WHERE flag <> 3 ORDER BY id;

--! update_realmlist_status
INSERT INTO realmlist(id, flag)
VALUES (:id, :flag)
ON CONFLICT (id)
DO UPDATE SET flag = EXCLUDED.flag;


--! set_uptime
INSERT INTO uptime (realmid, starttime, uptime, maxplayers, revision)
VALUES (:id, :start, :uptime, :population, :version)
ON CONFLICT (realmid)
DO UPDATE SET
    uptime = EXCLUDED.uptime,
    maxplayers = EXCLUDED.maxplayers;
