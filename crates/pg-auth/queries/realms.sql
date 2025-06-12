--! refresh_realmlist
SELECT id, name, icon as realm_type, gamebuild as build, address as external_address, local_address, local_subnet_mask, port, flag as flags, timezone, population FROM realmlist WHERE flag <> 3 ORDER BY id;

--! update_realmlist_status
update realmlist set flag = :flag where id = :id;

--! create_realm
insert into realmlist (name, address, local_address, local_subnet_mask, port) values (:name, :address, :local_address, :local_subnet_mask, :port) returning *;

--! set_uptime
INSERT INTO uptime (realmid, starttime, uptime, maxplayers, revision)
VALUES (:id, :start, :uptime, :maxplayers, :version)
ON CONFLICT (realmid)
DO UPDATE SET
    uptime = EXCLUDED.uptime,
    maxplayers = EXCLUDED.maxplayers;
