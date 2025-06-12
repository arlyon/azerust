CREATE EXTENSION uint;

CREATE TABLE account_data (
    account_id uint4 NOT NULL DEFAULT 0,
    type SMALLINT NOT NULL DEFAULT 0,
    time uint4 NOT NULL DEFAULT 0,
    data BYTEA NOT NULL,
    PRIMARY KEY (account_id, type)
);

CREATE TABLE account_instance_times (
    account_id INT NOT NULL,
    instance_id INT NOT NULL DEFAULT 0,
    release_time BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, instance_id)
);

CREATE TABLE account_tutorial (
    account_id INT NOT NULL DEFAULT 0,
    tut0 INT NOT NULL DEFAULT 0,
    tut1 INT NOT NULL DEFAULT 0,
    tut2 INT NOT NULL DEFAULT 0,
    tut3 INT NOT NULL DEFAULT 0,
    tut4 INT NOT NULL DEFAULT 0,
    tut5 INT NOT NULL DEFAULT 0,
    tut6 INT NOT NULL DEFAULT 0,
    tut7 INT NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id)
);

CREATE TABLE addons (
    name VARCHAR(120) NOT NULL DEFAULT '',
    crc INT NOT NULL DEFAULT 0,
    PRIMARY KEY (name)
);

CREATE TABLE arena_team (
    arena_team_id INT NOT NULL DEFAULT 0,
    name VARCHAR(24) NOT NULL,
    captain_guid INT NOT NULL DEFAULT 0,
    type SMALLINT NOT NULL DEFAULT 0,
    rating SMALLINT NOT NULL DEFAULT 0,
    season_games SMALLINT NOT NULL DEFAULT 0,
    season_wins SMALLINT NOT NULL DEFAULT 0,
    week_games SMALLINT NOT NULL DEFAULT 0,
    week_wins SMALLINT NOT NULL DEFAULT 0,
    rank INT NOT NULL DEFAULT 0,
    background_color INT NOT NULL DEFAULT 0,
    emblem_style SMALLINT NOT NULL DEFAULT 0,
    emblem_color INT NOT NULL DEFAULT 0,
    border_style SMALLINT NOT NULL DEFAULT 0,
    border_color INT NOT NULL DEFAULT 0,
    PRIMARY KEY (arena_team_id)
);

CREATE TABLE arena_team_member (
    arena_team_id INT NOT NULL DEFAULT 0,
    guid uint4 NOT NULL DEFAULT 0,
    week_games SMALLINT NOT NULL DEFAULT 0,
    week_wins SMALLINT NOT NULL DEFAULT 0,
    season_games SMALLINT NOT NULL DEFAULT 0,
    season_wins SMALLINT NOT NULL DEFAULT 0,
    personal_rating SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (arena_team_id, guid)
);

CREATE TABLE auctionbidders (
    id INT NOT NULL DEFAULT 0,
    bidder_guid INT NOT NULL DEFAULT 0,
    PRIMARY KEY (id, bidder_guid)
);

CREATE TABLE auctionhouse (
    id INT NOT NULL DEFAULT 0,
    house_id SMALLINT NOT NULL DEFAULT 7,
    item_guid INT NOT NULL DEFAULT 0,
    item_owner INT NOT NULL DEFAULT 0,
    buyout_price INT NOT NULL DEFAULT 0,
    time INT NOT NULL DEFAULT 0,
    buy_guid INT NOT NULL DEFAULT 0,
    last_bid INT NOT NULL DEFAULT 0,
    start_bid INT NOT NULL DEFAULT 0,
    deposit INT NOT NULL DEFAULT 0,
    flags SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    UNIQUE (item_guid)
);

CREATE TABLE banned_addons (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(255) NOT NULL DEFAULT '',
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE battleground_deserters (
    guid INT NOT NULL,
    type SMALLINT NOT NULL,
    datetime TIMESTAMP NOT NULL
);

CREATE TABLE bugreport (
    id SERIAL PRIMARY KEY,
    type TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE calendar_events (
    id BIGINT NOT NULL DEFAULT 0,
    creator INT NOT NULL DEFAULT 0,
    title VARCHAR(255) NOT NULL DEFAULT '',
    description VARCHAR(255) NOT NULL DEFAULT '',
    type SMALLINT NOT NULL DEFAULT 4,
    dungeon INT NOT NULL DEFAULT -1,
    event_time INT NOT NULL DEFAULT 0,
    flags INT NOT NULL DEFAULT 0,
    time2 INT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TABLE calendar_invites (
    id BIGINT NOT NULL DEFAULT 0,
    event BIGINT NOT NULL DEFAULT 0,
    invitee INT NOT NULL DEFAULT 0,
    sender INT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    status_time INT NOT NULL DEFAULT 0,
    rank SMALLINT NOT NULL DEFAULT 0,
    text VARCHAR(255) NOT NULL DEFAULT '',
    PRIMARY KEY (id)
);

CREATE TABLE channels (
    name VARCHAR(128) NOT NULL,
    team INT NOT NULL,
    announce SMALLINT NOT NULL DEFAULT 1,
    ownership SMALLINT NOT NULL DEFAULT 1,
    password VARCHAR(32),
    banned_list TEXT,
    last_used INT NOT NULL,
    PRIMARY KEY (name, team)
);

CREATE TABLE character_account_data (
    guid uint4 NOT NULL DEFAULT 0,
    type SMALLINT NOT NULL DEFAULT 0,
    time uint4 NOT NULL DEFAULT 0,
    data BYTEA NOT NULL,
    PRIMARY KEY (guid, type)
);

CREATE TABLE character_achievement (
    guid INT NOT NULL,
    achievement SMALLINT NOT NULL,
    date INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, achievement)
);

CREATE TABLE character_achievement_progress (
    guid INT NOT NULL,
    criteria SMALLINT NOT NULL,
    counter INT NOT NULL,
    date INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, criteria)
);

CREATE TABLE character_action (
    guid INT NOT NULL DEFAULT 0,
    spec SMALLINT NOT NULL DEFAULT 0,
    button SMALLINT NOT NULL DEFAULT 0,
    action INT NOT NULL DEFAULT 0,
    type SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spec, button)
);

CREATE TABLE character_arena_stats (
    guid INT NOT NULL DEFAULT 0,
    slot SMALLINT NOT NULL DEFAULT 0,
    match_maker_rating SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, slot)
);

CREATE TABLE character_aura (
    guid INT NOT NULL DEFAULT 0,
    caster_guid BIGINT NOT NULL DEFAULT 0,
    item_guid BIGINT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    effect_mask SMALLINT NOT NULL DEFAULT 0,
    recalculate_mask SMALLINT NOT NULL DEFAULT 0,
    stack_count SMALLINT NOT NULL DEFAULT 1,
    amount0 INT NOT NULL DEFAULT 0,
    amount1 INT NOT NULL DEFAULT 0,
    amount2 INT NOT NULL DEFAULT 0,
    base_amount0 INT NOT NULL DEFAULT 0,
    base_amount1 INT NOT NULL DEFAULT 0,
    base_amount2 INT NOT NULL DEFAULT 0,
    max_duration INT NOT NULL DEFAULT 0,
    remain_time INT NOT NULL DEFAULT 0,
    remain_charges SMALLINT NOT NULL DEFAULT 0,
    crit_chance FLOAT NOT NULL DEFAULT 0,
    apply_resilience SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, caster_guid, item_guid, spell, effect_mask)
);

CREATE TABLE character_banned (
    guid INT NOT NULL DEFAULT 0,
    ban_date INT NOT NULL DEFAULT 0,
    unban_date INT NOT NULL DEFAULT 0,
    banned_by VARCHAR(50) NOT NULL,
    ban_reason VARCHAR(255) NOT NULL,
    active SMALLINT NOT NULL DEFAULT 1,
    PRIMARY KEY (guid, ban_date)
);

CREATE TABLE character_battleground_data (
    guid INT NOT NULL DEFAULT 0,
    instance_id INT NOT NULL,
    team SMALLINT NOT NULL,
    join_x FLOAT NOT NULL DEFAULT 0,
    join_y FLOAT NOT NULL DEFAULT 0,
    join_z FLOAT NOT NULL DEFAULT 0,
    join_o FLOAT NOT NULL DEFAULT 0,
    join_map_id SMALLINT NOT NULL DEFAULT 0,
    taxi_start INT NOT NULL DEFAULT 0,
    taxi_end INT NOT NULL DEFAULT 0,
    mount_spell INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE character_battleground_random (guid INT NOT NULL DEFAULT 0, PRIMARY KEY (guid));

CREATE TABLE character_declinedname (
    guid INT NOT NULL DEFAULT 0,
    genitive VARCHAR(15) NOT NULL DEFAULT '',
    dative VARCHAR(15) NOT NULL DEFAULT '',
    accusative VARCHAR(15) NOT NULL DEFAULT '',
    instrumental VARCHAR(15) NOT NULL DEFAULT '',
    prepositional VARCHAR(15) NOT NULL DEFAULT '',
    PRIMARY KEY (guid)
);

CREATE TABLE character_equipmentsets (
    guid INT NOT NULL DEFAULT 0,
    set_guid BIGSERIAL PRIMARY KEY,
    set_index SMALLINT NOT NULL DEFAULT 0,
    name VARCHAR(31) NOT NULL,
    icon_name VARCHAR(100) NOT NULL,
    ignore_mask INT NOT NULL DEFAULT 0,
    item0 INT NOT NULL DEFAULT 0,
    item1 INT NOT NULL DEFAULT 0,
    item2 INT NOT NULL DEFAULT 0,
    item3 INT NOT NULL DEFAULT 0,
    item4 INT NOT NULL DEFAULT 0,
    item5 INT NOT NULL DEFAULT 0,
    item6 INT NOT NULL DEFAULT 0,
    item7 INT NOT NULL DEFAULT 0,
    item8 INT NOT NULL DEFAULT 0,
    item9 INT NOT NULL DEFAULT 0,
    item10 INT NOT NULL DEFAULT 0,
    item11 INT NOT NULL DEFAULT 0,
    item12 INT NOT NULL DEFAULT 0,
    item13 INT NOT NULL DEFAULT 0,
    item14 INT NOT NULL DEFAULT 0,
    item15 INT NOT NULL DEFAULT 0,
    item16 INT NOT NULL DEFAULT 0,
    item17 INT NOT NULL DEFAULT 0,
    item18 INT NOT NULL DEFAULT 0,
    UNIQUE (guid, set_guid, set_index)
);

CREATE TABLE character_fishingsteps (
    guid INT NOT NULL DEFAULT 0,
    fishing_steps SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE character_gifts (
    guid INT NOT NULL DEFAULT 0,
    item_guid INT NOT NULL DEFAULT 0,
    entry INT NOT NULL DEFAULT 0,
    flags INT NOT NULL DEFAULT 0,
    PRIMARY KEY (item_guid)
);

CREATE TABLE character_glyphs (
    guid INT NOT NULL,
    talent_group SMALLINT NOT NULL DEFAULT 0,
    glyph1 SMALLINT DEFAULT 0,
    glyph2 SMALLINT DEFAULT 0,
    glyph3 SMALLINT DEFAULT 0,
    glyph4 SMALLINT DEFAULT 0,
    glyph5 SMALLINT DEFAULT 0,
    glyph6 SMALLINT DEFAULT 0,
    PRIMARY KEY (guid, talent_group)
);

CREATE TABLE character_homebind (
    guid INT NOT NULL DEFAULT 0,
    map_id SMALLINT NOT NULL DEFAULT 0,
    zone_id SMALLINT NOT NULL DEFAULT 0,
    pos_x FLOAT NOT NULL DEFAULT 0,
    pos_y FLOAT NOT NULL DEFAULT 0,
    pos_z FLOAT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE character_instance (
    guid INT NOT NULL DEFAULT 0,
    instance INT NOT NULL DEFAULT 0,
    permanent SMALLINT NOT NULL DEFAULT 0,
    extend_state SMALLINT NOT NULL DEFAULT 1,
    PRIMARY KEY (guid, instance)
);

CREATE TABLE character_inventory (
    guid INT NOT NULL DEFAULT 0,
    bag INT NOT NULL DEFAULT 0,
    slot SMALLINT NOT NULL DEFAULT 0,
    item INT NOT NULL DEFAULT 0,
    PRIMARY KEY (item),
    UNIQUE (guid, bag, slot)
);

CREATE TABLE character_pet (
    id INT NOT NULL DEFAULT 0,
    entry INT NOT NULL DEFAULT 0,
    owner INT NOT NULL DEFAULT 0,
    model_id INT DEFAULT 0,
    created_by_spell INT NOT NULL DEFAULT 0,
    pet_type SMALLINT NOT NULL DEFAULT 0,
    level SMALLINT NOT NULL DEFAULT 1,
    exp INT NOT NULL DEFAULT 0,
    react_state SMALLINT NOT NULL DEFAULT 0,
    name VARCHAR(21) NOT NULL DEFAULT 'Pet',
    renamed SMALLINT NOT NULL DEFAULT 0,
    slot SMALLINT NOT NULL DEFAULT 0,
    cur_health INT NOT NULL DEFAULT 1,
    cur_mana INT NOT NULL DEFAULT 0,
    cur_happiness INT NOT NULL DEFAULT 0,
    save_time INT NOT NULL DEFAULT 0,
    ab_data TEXT,
    PRIMARY KEY (id)
);

CREATE TABLE character_pet_declinedname (
    id INT NOT NULL DEFAULT 0,
    owner INT NOT NULL DEFAULT 0,
    genitive VARCHAR(12) NOT NULL DEFAULT '',
    dative VARCHAR(12) NOT NULL DEFAULT '',
    accusative VARCHAR(12) NOT NULL DEFAULT '',
    instrumental VARCHAR(12) NOT NULL DEFAULT '',
    prepositional VARCHAR(12) NOT NULL DEFAULT '',
    PRIMARY KEY (id)
);

CREATE TABLE character_queststatus (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    status SMALLINT NOT NULL DEFAULT 0,
    explored SMALLINT NOT NULL DEFAULT 0,
    timer INT NOT NULL DEFAULT 0,
    mob_count1 SMALLINT NOT NULL DEFAULT 0,
    mob_count2 SMALLINT NOT NULL DEFAULT 0,
    mob_count3 SMALLINT NOT NULL DEFAULT 0,
    mob_count4 SMALLINT NOT NULL DEFAULT 0,
    item_count1 SMALLINT NOT NULL DEFAULT 0,
    item_count2 SMALLINT NOT NULL DEFAULT 0,
    item_count3 SMALLINT NOT NULL DEFAULT 0,
    item_count4 SMALLINT NOT NULL DEFAULT 0,
    item_count5 SMALLINT NOT NULL DEFAULT 0,
    item_count6 SMALLINT NOT NULL DEFAULT 0,
    player_count SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_queststatus_daily (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    time INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_queststatus_monthly (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_queststatus_rewarded (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    active SMALLINT NOT NULL DEFAULT 1,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_queststatus_seasonal (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    event INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_queststatus_weekly (
    guid INT NOT NULL DEFAULT 0,
    quest INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, quest)
);

CREATE TABLE character_reputation (
    guid INT NOT NULL DEFAULT 0,
    faction SMALLINT NOT NULL DEFAULT 0,
    standing INT NOT NULL DEFAULT 0,
    flags SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, faction)
);

CREATE TABLE character_skills (
    guid INT NOT NULL,
    skill SMALLINT NOT NULL,
    value SMALLINT NOT NULL,
    max SMALLINT NOT NULL,
    PRIMARY KEY (guid, skill)
);

CREATE TABLE character_social (
    guid uint4 NOT NULL DEFAULT 0,
    friend uint4 NOT NULL DEFAULT 0,
    flags uint4 NOT NULL DEFAULT 0,
    note VARCHAR(48) NOT NULL DEFAULT '',
    PRIMARY KEY (guid, friend, flags)
);

CREATE TABLE character_spell (
    guid INT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    active SMALLINT NOT NULL DEFAULT 1,
    disabled SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spell)
);

CREATE TABLE character_spell_cooldown (
    guid INT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    item INT NOT NULL DEFAULT 0,
    time INT NOT NULL DEFAULT 0,
    category_id INT NOT NULL DEFAULT 0,
    category_end INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spell)
);

CREATE TABLE character_stats (
    guid INT NOT NULL DEFAULT 0,
    max_health INT NOT NULL DEFAULT 0,
    max_power1 INT NOT NULL DEFAULT 0,
    max_power2 INT NOT NULL DEFAULT 0,
    max_power3 INT NOT NULL DEFAULT 0,
    max_power4 INT NOT NULL DEFAULT 0,
    max_power5 INT NOT NULL DEFAULT 0,
    max_power6 INT NOT NULL DEFAULT 0,
    max_power7 INT NOT NULL DEFAULT 0,
    strength INT NOT NULL DEFAULT 0,
    agility INT NOT NULL DEFAULT 0,
    stamina INT NOT NULL DEFAULT 0,
    intellect INT NOT NULL DEFAULT 0,
    spirit INT NOT NULL DEFAULT 0,
    armor INT NOT NULL DEFAULT 0,
    res_holy INT NOT NULL DEFAULT 0,
    res_fire INT NOT NULL DEFAULT 0,
    res_nature INT NOT NULL DEFAULT 0,
    res_frost INT NOT NULL DEFAULT 0,
    res_shadow INT NOT NULL DEFAULT 0,
    res_arcane INT NOT NULL DEFAULT 0,
    block_pct FLOAT NOT NULL DEFAULT 0,
    dodge_pct FLOAT NOT NULL DEFAULT 0,
    parry_pct FLOAT NOT NULL DEFAULT 0,
    crit_pct FLOAT NOT NULL DEFAULT 0,
    ranged_crit_pct FLOAT NOT NULL DEFAULT 0,
    spell_crit_pct FLOAT NOT NULL DEFAULT 0,
    attack_power INT NOT NULL DEFAULT 0,
    ranged_attack_power INT NOT NULL DEFAULT 0,
    spell_power INT NOT NULL DEFAULT 0,
    resilience INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE character_talent (
    guid INT NOT NULL,
    spell INT NOT NULL,
    talent_group SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spell, talent_group)
);

CREATE TABLE characters (
    guid uint4 NOT NULL DEFAULT 0,
    account uint4 NOT NULL DEFAULT 0,
    name VARCHAR(12),
    race uint1 NOT NULL DEFAULT 0,
    class uint1 NOT NULL DEFAULT 0,
    gender uint1 NOT NULL DEFAULT 0,
    level uint1 NOT NULL DEFAULT 0,
    xp INT NOT NULL DEFAULT 0,
    money INT NOT NULL DEFAULT 0,
    skin uint1 NOT NULL DEFAULT 0,
    face uint1 NOT NULL DEFAULT 0,
    hair_style uint1 NOT NULL DEFAULT 0,
    hair_color uint1 NOT NULL DEFAULT 0,
    facial_style uint1 NOT NULL DEFAULT 0,
    bank_slots SMALLINT NOT NULL DEFAULT 0,
    rest_state SMALLINT NOT NULL DEFAULT 0,
    player_flags INT NOT NULL DEFAULT 0,
    position_x REAL NOT NULL DEFAULT 0,
    position_y REAL NOT NULL DEFAULT 0,
    position_z REAL NOT NULL DEFAULT 0,
    map uint2 NOT NULL DEFAULT 0,
    instance_id INT NOT NULL DEFAULT 0,
    instance_mode_mask SMALLINT NOT NULL DEFAULT 0,
    orientation FLOAT NOT NULL DEFAULT 0,
    taxi_mask TEXT NOT NULL,
    online bool NOT NULL DEFAULT false,
    cinematic SMALLINT NOT NULL DEFAULT 0,
    total_time INT NOT NULL DEFAULT 0,
    level_time INT NOT NULL DEFAULT 0,
    logout_time INT NOT NULL DEFAULT 0,
    is_logout_resting SMALLINT NOT NULL DEFAULT 0,
    rest_bonus FLOAT NOT NULL DEFAULT 0,
    reset_talents_cost INT NOT NULL DEFAULT 0,
    reset_talents_time INT NOT NULL DEFAULT 0,
    trans_x FLOAT NOT NULL DEFAULT 0,
    trans_y FLOAT NOT NULL DEFAULT 0,
    trans_z FLOAT NOT NULL DEFAULT 0,
    trans_o FLOAT NOT NULL DEFAULT 0,
    trans_guid INT NOT NULL DEFAULT 0,
    extra_flags SMALLINT NOT NULL DEFAULT 0,
    stable_slots SMALLINT NOT NULL DEFAULT 0,
    at_login SMALLINT NOT NULL DEFAULT 0,
    zone uint2 NOT NULL DEFAULT 0,
    death_expire_time INT NOT NULL DEFAULT 0,
    taxi_path TEXT,
    arena_points INT NOT NULL DEFAULT 0,
    total_honor_points INT NOT NULL DEFAULT 0,
    today_honor_points INT NOT NULL DEFAULT 0,
    yesterday_honor_points INT NOT NULL DEFAULT 0,
    total_kills INT NOT NULL DEFAULT 0,
    today_kills SMALLINT NOT NULL DEFAULT 0,
    yesterday_kills SMALLINT NOT NULL DEFAULT 0,
    chosen_title INT NOT NULL DEFAULT 0,
    known_currencies BIGINT NOT NULL DEFAULT 0,
    watched_faction INT NOT NULL DEFAULT 0,
    drunk SMALLINT NOT NULL DEFAULT 0,
    health INT NOT NULL DEFAULT 0,
    power1 INT NOT NULL DEFAULT 0,
    power2 INT NOT NULL DEFAULT 0,
    power3 INT NOT NULL DEFAULT 0,
    power4 INT NOT NULL DEFAULT 0,
    power5 INT NOT NULL DEFAULT 0,
    power6 INT NOT NULL DEFAULT 0,
    power7 INT NOT NULL DEFAULT 0,
    latency INT NOT NULL DEFAULT 0,
    talent_groups_count SMALLINT NOT NULL DEFAULT 1,
    active_talent_group SMALLINT NOT NULL DEFAULT 0,
    explored_zones TEXT,
    equipment_cache TEXT,
    ammo_id INT NOT NULL DEFAULT 0,
    known_titles TEXT,
    action_bars SMALLINT NOT NULL DEFAULT 0,
    grantable_levels SMALLINT NOT NULL DEFAULT 0,
    delete_infos_account INT,
    delete_infos_name VARCHAR(12),
    delete_date INT,
    PRIMARY KEY (guid),
    UNIQUE (name)
);

CREATE TABLE corpse (
    guid INT NOT NULL DEFAULT 0,
    pos_x FLOAT NOT NULL DEFAULT 0,
    pos_y FLOAT NOT NULL DEFAULT 0,
    pos_z FLOAT NOT NULL DEFAULT 0,
    orientation FLOAT NOT NULL DEFAULT 0,
    map_id SMALLINT NOT NULL DEFAULT 0,
    phase_mask INT NOT NULL DEFAULT 1,
    display_id INT NOT NULL DEFAULT 0,
    item_cache TEXT NOT NULL,
    bytes1 INT NOT NULL DEFAULT 0,
    bytes2 INT NOT NULL DEFAULT 0,
    guild_id INT NOT NULL DEFAULT 0,
    flags SMALLINT NOT NULL DEFAULT 0,
    dyn_flags SMALLINT NOT NULL DEFAULT 0,
    time INT NOT NULL DEFAULT 0,
    corpse_type SMALLINT NOT NULL DEFAULT 0,
    instance_id INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE game_event_condition_save (
    event_entry SMALLINT NOT NULL,
    condition_id INT NOT NULL DEFAULT 0,
    done FLOAT DEFAULT 0,
    PRIMARY KEY (event_entry, condition_id)
);

CREATE TABLE game_event_save (
    event_entry SMALLINT NOT NULL,
    state SMALLINT NOT NULL DEFAULT 1,
    next_start INT NOT NULL DEFAULT 0,
    PRIMARY KEY (event_entry)
);

CREATE TABLE gm_subsurvey (
    survey_id SERIAL,
    question_id INT NOT NULL DEFAULT 0,
    answer INT NOT NULL DEFAULT 0,
    answer_comment TEXT NOT NULL,
    PRIMARY KEY (survey_id, question_id)
);

CREATE TABLE gm_survey (
    survey_id SERIAL PRIMARY KEY,
    guid INT NOT NULL DEFAULT 0,
    main_survey INT NOT NULL DEFAULT 0,
    comment TEXT NOT NULL,
    create_time INT NOT NULL DEFAULT 0
);

CREATE TABLE gm_ticket (
    id SERIAL PRIMARY KEY,
    type SMALLINT NOT NULL DEFAULT 0,
    player_guid INT NOT NULL DEFAULT 0,
    name VARCHAR(12) NOT NULL,
    description TEXT NOT NULL,
    create_time INT NOT NULL DEFAULT 0,
    map_id SMALLINT NOT NULL DEFAULT 0,
    pos_x FLOAT NOT NULL DEFAULT 0,
    pos_y FLOAT NOT NULL DEFAULT 0,
    pos_z FLOAT NOT NULL DEFAULT 0,
    last_modified_time INT NOT NULL DEFAULT 0,
    closed_by INT NOT NULL DEFAULT 0,
    assigned_to INT NOT NULL DEFAULT 0,
    comment TEXT NOT NULL,
    response TEXT NOT NULL,
    completed SMALLINT NOT NULL DEFAULT 0,
    escalated SMALLINT NOT NULL DEFAULT 0,
    viewed SMALLINT NOT NULL DEFAULT 0,
    need_more_help SMALLINT NOT NULL DEFAULT 0,
    resolved_by INT NOT NULL DEFAULT 0
);

CREATE TABLE group_instance (
    guid INT NOT NULL DEFAULT 0,
    instance INT NOT NULL DEFAULT 0,
    permanent SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, instance)
);

CREATE TABLE group_member (
    guid INT NOT NULL,
    member_guid INT NOT NULL,
    member_flags SMALLINT NOT NULL DEFAULT 0,
    subgroup SMALLINT NOT NULL DEFAULT 0,
    roles SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (member_guid)
);

CREATE TABLE groups (
    guid INT NOT NULL,
    leader_guid INT NOT NULL,
    loot_method SMALLINT NOT NULL,
    looter_guid INT NOT NULL,
    loot_threshold SMALLINT NOT NULL,
    icon1 BIGINT NOT NULL,
    icon2 BIGINT NOT NULL,
    icon3 BIGINT NOT NULL,
    icon4 BIGINT NOT NULL,
    icon5 BIGINT NOT NULL,
    icon6 BIGINT NOT NULL,
    icon7 BIGINT NOT NULL,
    icon8 BIGINT NOT NULL,
    group_type SMALLINT NOT NULL,
    difficulty SMALLINT NOT NULL DEFAULT 0,
    raid_difficulty SMALLINT NOT NULL DEFAULT 0,
    master_looter_guid INT NOT NULL,
    PRIMARY KEY (guid)
);

CREATE TABLE guild (
    guild_id INT NOT NULL DEFAULT 0,
    name VARCHAR(24) NOT NULL DEFAULT '',
    leader_guid INT NOT NULL DEFAULT 0,
    emblem_style SMALLINT NOT NULL DEFAULT 0,
    emblem_color SMALLINT NOT NULL DEFAULT 0,
    border_style SMALLINT NOT NULL DEFAULT 0,
    border_color SMALLINT NOT NULL DEFAULT 0,
    background_color SMALLINT NOT NULL DEFAULT 0,
    info VARCHAR(500) NOT NULL DEFAULT '',
    motd VARCHAR(128) NOT NULL DEFAULT '',
    create_date INT NOT NULL DEFAULT 0,
    bank_money BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id)
);

CREATE TABLE guild_bank_eventlog (
    guild_id INT NOT NULL DEFAULT 0,
    log_guid INT NOT NULL DEFAULT 0,
    tab_id SMALLINT NOT NULL DEFAULT 0,
    event_type SMALLINT NOT NULL DEFAULT 0,
    player_guid INT NOT NULL DEFAULT 0,
    item_or_money INT NOT NULL DEFAULT 0,
    item_stack_count SMALLINT NOT NULL DEFAULT 0,
    dest_tab_id SMALLINT NOT NULL DEFAULT 0,
    timestamp INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, log_guid, tab_id)
);

CREATE TABLE guild_bank_item (
    guild_id INT NOT NULL DEFAULT 0,
    tab_id SMALLINT NOT NULL DEFAULT 0,
    slot_id SMALLINT NOT NULL DEFAULT 0,
    item_guid INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, tab_id, slot_id)
);

CREATE TABLE guild_bank_right (
    guild_id INT NOT NULL DEFAULT 0,
    tab_id SMALLINT NOT NULL DEFAULT 0,
    rid SMALLINT NOT NULL DEFAULT 0,
    gb_right SMALLINT NOT NULL DEFAULT 0,
    slot_per_day INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, tab_id, rid)
);

CREATE TABLE guild_bank_tab (
    guild_id INT NOT NULL DEFAULT 0,
    tab_id SMALLINT NOT NULL DEFAULT 0,
    tab_name VARCHAR(16) NOT NULL DEFAULT '',
    tab_icon VARCHAR(100) NOT NULL DEFAULT '',
    tab_text VARCHAR(500),
    PRIMARY KEY (guild_id, tab_id)
);

CREATE TABLE guild_eventlog (
    guild_id INT NOT NULL,
    log_guid INT NOT NULL,
    event_type SMALLINT NOT NULL,
    player_guid1 INT NOT NULL,
    player_guid2 INT NOT NULL,
    new_rank SMALLINT NOT NULL,
    timestamp INT NOT NULL
);

CREATE TABLE guild_member (
    guild_id INT NOT NULL,
    guid INT NOT NULL,
    rank SMALLINT NOT NULL,
    pnote VARCHAR(31) NOT NULL DEFAULT '',
    offnote VARCHAR(31) NOT NULL DEFAULT '',
    UNIQUE (guid)
);

CREATE TABLE guild_member_withdraw (
    guid INT NOT NULL,
    tab0 INT NOT NULL DEFAULT 0,
    tab1 INT NOT NULL DEFAULT 0,
    tab2 INT NOT NULL DEFAULT 0,
    tab3 INT NOT NULL DEFAULT 0,
    tab4 INT NOT NULL DEFAULT 0,
    tab5 INT NOT NULL DEFAULT 0,
    money INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE guild_rank (
    guild_id INT NOT NULL DEFAULT 0,
    rid SMALLINT NOT NULL,
    rname VARCHAR(20) NOT NULL DEFAULT '',
    rights INT NOT NULL DEFAULT 0,
    bank_money_per_day INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, rid)
);

CREATE TABLE instance (
    id INT NOT NULL DEFAULT 0,
    map SMALLINT NOT NULL DEFAULT 0,
    reset_time BIGINT NOT NULL DEFAULT 0,
    difficulty SMALLINT NOT NULL DEFAULT 0,
    completed_encounters INT NOT NULL DEFAULT 0,
    data TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE instance_reset (
    map_id SMALLINT NOT NULL DEFAULT 0,
    difficulty SMALLINT NOT NULL DEFAULT 0,
    reset_time BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (map_id, difficulty)
);

CREATE TABLE item_instance (
    guid INT NOT NULL DEFAULT 0,
    item_entry SMALLINT NOT NULL DEFAULT 0,
    owner_guid INT NOT NULL DEFAULT 0,
    creator_guid INT NOT NULL DEFAULT 0,
    gift_creator_guid INT NOT NULL DEFAULT 0,
    count INT NOT NULL DEFAULT 1,
    duration INT NOT NULL DEFAULT 0,
    charges TEXT,
    flags INT NOT NULL DEFAULT 0,
    enchantments TEXT NOT NULL,
    random_property_id SMALLINT NOT NULL DEFAULT 0,
    durability SMALLINT NOT NULL DEFAULT 0,
    played_time INT NOT NULL DEFAULT 0,
    text TEXT,
    PRIMARY KEY (guid)
);

CREATE TABLE item_loot_items (
    container_id INT NOT NULL DEFAULT 0,
    item_id INT NOT NULL DEFAULT 0,
    item_count INT NOT NULL DEFAULT 0,
    item_index INT NOT NULL DEFAULT 0,
    follow_rules SMALLINT NOT NULL DEFAULT 0,
    ffa SMALLINT NOT NULL DEFAULT 0,
    blocked SMALLINT NOT NULL DEFAULT 0,
    counted SMALLINT NOT NULL DEFAULT 0,
    under_threshold SMALLINT NOT NULL DEFAULT 0,
    needs_quest SMALLINT NOT NULL DEFAULT 0,
    rnd_prop INT NOT NULL DEFAULT 0,
    rnd_suffix INT NOT NULL DEFAULT 0,
    PRIMARY KEY (container_id, item_id)
);

CREATE TABLE item_loot_money (
    container_id INT NOT NULL DEFAULT 0,
    money INT NOT NULL DEFAULT 0,
    PRIMARY KEY (container_id)
);

CREATE TABLE item_refund_instance (
    item_guid INT NOT NULL,
    player_guid INT NOT NULL,
    paid_money INT NOT NULL DEFAULT 0,
    paid_extended_cost SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (item_guid, player_guid)
);

CREATE TABLE item_soulbound_trade_data (
    item_guid INT NOT NULL,
    allowed_players TEXT NOT NULL,
    PRIMARY KEY (item_guid)
);

CREATE TABLE lag_reports (
    report_id SERIAL PRIMARY KEY,
    guid INT NOT NULL DEFAULT 0,
    lag_type SMALLINT NOT NULL DEFAULT 0,
    map_id SMALLINT NOT NULL DEFAULT 0,
    pos_x FLOAT NOT NULL DEFAULT 0,
    pos_y FLOAT NOT NULL DEFAULT 0,
    pos_z FLOAT NOT NULL DEFAULT 0,
    latency INT NOT NULL DEFAULT 0,
    create_time INT NOT NULL DEFAULT 0
);

CREATE TABLE lfg_data (
    guid INT NOT NULL DEFAULT 0,
    dungeon INT NOT NULL DEFAULT 0,
    state SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid)
);

CREATE TABLE mail (
    id INT NOT NULL DEFAULT 0,
    message_type SMALLINT NOT NULL DEFAULT 0,
    stationery SMALLINT NOT NULL DEFAULT 41,
    mail_template_id SMALLINT NOT NULL DEFAULT 0,
    sender INT NOT NULL DEFAULT 0,
    receiver INT NOT NULL DEFAULT 0,
    subject TEXT,
    body TEXT,
    has_items SMALLINT NOT NULL DEFAULT 0,
    expire_time INT NOT NULL DEFAULT 0,
    deliver_time INT NOT NULL DEFAULT 0,
    money INT NOT NULL DEFAULT 0,
    cod INT NOT NULL DEFAULT 0,
    checked SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (id)
);

CREATE TABLE mail_items (
    mail_id INT NOT NULL DEFAULT 0,
    item_guid INT NOT NULL DEFAULT 0,
    receiver INT NOT NULL DEFAULT 0,
    PRIMARY KEY (item_guid)
);

CREATE TABLE pet_aura (
    guid INT NOT NULL DEFAULT 0,
    caster_guid BIGINT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    effect_mask SMALLINT NOT NULL DEFAULT 0,
    recalculate_mask SMALLINT NOT NULL DEFAULT 0,
    stack_count SMALLINT NOT NULL DEFAULT 1,
    amount0 INT NOT NULL,
    amount1 INT NOT NULL,
    amount2 INT NOT NULL,
    base_amount0 INT NOT NULL,
    base_amount1 INT NOT NULL,
    base_amount2 INT NOT NULL,
    max_duration INT NOT NULL DEFAULT 0,
    remain_time INT NOT NULL DEFAULT 0,
    remain_charges SMALLINT NOT NULL DEFAULT 0,
    crit_chance FLOAT NOT NULL DEFAULT 0,
    apply_resilience SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, caster_guid, spell, effect_mask)
);

CREATE TABLE pet_spell (
    guid INT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    active SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spell)
);

CREATE TABLE pet_spell_cooldown (
    guid INT NOT NULL DEFAULT 0,
    spell INT NOT NULL DEFAULT 0,
    time INT NOT NULL DEFAULT 0,
    category_id INT NOT NULL DEFAULT 0,
    category_end INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guid, spell)
);

CREATE TABLE petition (
    owner_guid INT NOT NULL,
    petition_guid INT DEFAULT 0,
    name VARCHAR(24) NOT NULL,
    type SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (owner_guid, type),
    UNIQUE (owner_guid, petition_guid)
);

CREATE TABLE petition_sign (
    owner_guid INT NOT NULL,
    petition_guid INT NOT NULL DEFAULT 0,
    player_guid INT NOT NULL DEFAULT 0,
    player_account INT NOT NULL DEFAULT 0,
    type SMALLINT NOT NULL DEFAULT 0,
    PRIMARY KEY (petition_guid, player_guid)
);

CREATE TABLE pool_quest_save (
    pool_id INT NOT NULL DEFAULT 0,
    quest_id INT NOT NULL DEFAULT 0,
    PRIMARY KEY (pool_id, quest_id)
);

CREATE TABLE pvpstats_battlegrounds (
    id BIGSERIAL PRIMARY KEY,
    winner_faction SMALLINT NOT NULL,
    bracket_id SMALLINT NOT NULL,
    type SMALLINT NOT NULL,
    date TIMESTAMP NOT NULL
);

CREATE TABLE pvpstats_players (
    battleground_id BIGINT NOT NULL,
    character_guid INT NOT NULL,
    winner BOOLEAN NOT NULL,
    score_killing_blows SMALLINT NOT NULL,
    score_deaths SMALLINT NOT NULL,
    score_honorable_kills SMALLINT NOT NULL,
    score_bonus_honor SMALLINT NOT NULL,
    score_damage_done INT NOT NULL,
    score_healing_done INT NOT NULL,
    attr_1 INT NOT NULL DEFAULT 0,
    attr_2 INT NOT NULL DEFAULT 0,
    attr_3 INT NOT NULL DEFAULT 0,
    attr_4 INT NOT NULL DEFAULT 0,
    attr_5 INT NOT NULL DEFAULT 0,
    PRIMARY KEY (battleground_id, character_guid)
);

CREATE TABLE quest_tracker (
    id SMALLINT NOT NULL DEFAULT 0,
    character_guid INT NOT NULL DEFAULT 0,
    quest_accept_time TIMESTAMP NOT NULL,
    quest_complete_time TIMESTAMP DEFAULT NULL,
    quest_abandon_time TIMESTAMP DEFAULT NULL,
    completed_by_gm SMALLINT NOT NULL DEFAULT 0,
    core_hash VARCHAR(120) NOT NULL DEFAULT '0',
    core_revision VARCHAR(120) NOT NULL DEFAULT '0'
);

CREATE TABLE reserved_name (name VARCHAR(12) NOT NULL, PRIMARY KEY (name));

CREATE TABLE respawn (
    type SMALLINT NOT NULL,
    spawn_id INT NOT NULL,
    respawn_time BIGINT NOT NULL,
    map_id SMALLINT NOT NULL,
    instance_id INT NOT NULL,
    PRIMARY KEY (type, spawn_id, instance_id)
);

CREATE TABLE updates (
    name VARCHAR(200) NOT NULL,
    hash CHAR(40) DEFAULT '',
    state TEXT NOT NULL DEFAULT 'RELEASED',
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    speed INT NOT NULL DEFAULT 0,
    PRIMARY KEY (name)
);

CREATE TABLE updates_include (
    path VARCHAR(200) NOT NULL,
    state TEXT NOT NULL DEFAULT 'RELEASED',
    PRIMARY KEY (path)
);

CREATE TABLE warden_action (
    warden_id SMALLINT NOT NULL,
    action SMALLINT DEFAULT NULL,
    PRIMARY KEY (warden_id)
);

CREATE TABLE worldstates (
    entry INT NOT NULL DEFAULT 0,
    value INT NOT NULL DEFAULT 0,
    comment TEXT,
    PRIMARY KEY (entry)
);
