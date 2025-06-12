--! get_character_by_id
SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where guid = :guid;

--! get_character_by_name
SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where name = :name;

--! get_character_by_account
SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where account = :account;

--! count_by_account
SELECT count(*) as count FROM characters where account = :account;

--! name_available
SELECT count(*) = 0 as available FROM characters where name = :name;

--! insert_character
INSERT INTO characters
    (account, guid, level, name, race, class, gender, skin, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z, taxi_mask, taxi_path, explored_zones, equipment_cache, known_titles)
values
    (:account, :guid, 1, :name, :race, :class, :gender, :skin, :face, :hair_style, :hair_color, :facial_style, :zone, :map, :position_x, :position_y, :position_z, '','', '', '', '')
returning
    guid;

--! delete_character
DELETE FROM characters where guid = :id;

--! account_data
SELECT type, time, data FROM account_data WHERE account_id = :id;

--! character_account_data
SELECT type, time, data FROM character_account_data WHERE guid = :id;

--! list_characters
SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters;

--! get_friends
select friend, flags, note from character_social where guid = :id;

--! add_friend_by_name (note?)
INSERT INTO character_social (guid, friend, note) values (:id, :friend_id, :note);
