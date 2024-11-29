-- Table: account
CREATE TABLE account (
    id SERIAL PRIMARY KEY,
    username VARCHAR(32) NOT NULL UNIQUE,
    salt BYTEA NOT NULL,
    verifier BYTEA NOT NULL,
    session_key_auth BYTEA,
    session_key_bnet BYTEA,
    totp_secret BYTEA,
    email VARCHAR(255) NOT NULL CHECK (email != ''),
    reg_mail VARCHAR(255) NOT NULL CHECK (reg_mail != ''),
    joindate TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_ip INET NOT NULL DEFAULT '127.0.0.1',
    last_attempt_ip INET NOT NULL DEFAULT '127.0.0.1',
    failed_logins INT NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    lock_country VARCHAR(2) NOT NULL DEFAULT '00',
    last_login TIMESTAMP NULL,
    online BOOLEAN NOT NULL DEFAULT FALSE,
    expansion SMALLINT NOT NULL DEFAULT 2,
    mutetime BIGINT NOT NULL DEFAULT 0 CHECK (mutetime >= 0),
    mutereason VARCHAR(255) NOT NULL DEFAULT '',
    muteby VARCHAR(50) NOT NULL DEFAULT '',
    locale SMALLINT NOT NULL DEFAULT 0,
    os VARCHAR(3) NOT NULL DEFAULT '',
    timezone_offset SMALLINT NOT NULL DEFAULT 0,
    recruiter INT NOT NULL DEFAULT 0
);

COMMENT ON TABLE account IS 'Account information for authentication.';

-- Table: account_access
CREATE TABLE account_access (
    account_id INT NOT NULL,
    security_level SMALLINT NOT NULL CHECK (security_level >= 0),
    realm_id INT NOT NULL DEFAULT -1,
    comment VARCHAR(255),
    PRIMARY KEY (account_id, realm_id)
);

COMMENT ON TABLE account_access IS 'Access levels for accounts per realm.';

-- Table: account_banned
CREATE TABLE account_banned (
    id INT NOT NULL,
    bandate TIMESTAMP NOT NULL,
    unbandate TIMESTAMP NOT NULL,
    bannedby VARCHAR(50) NOT NULL,
    banreason VARCHAR(255) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (id, bandate)
);

COMMENT ON TABLE account_banned IS 'Ban records for accounts.';

-- Table: autobroadcast
CREATE TABLE autobroadcast (
    realmid INT NOT NULL DEFAULT -1,
    id SERIAL PRIMARY KEY,
    weight SMALLINT DEFAULT 1 CHECK (weight > 0),
    text TEXT NOT NULL
);

COMMENT ON TABLE autobroadcast IS 'Automatic broadcast messages.';

-- Table: build_auth_key
CREATE TABLE build_auth_key (
    build INT NOT NULL,
    platform CHAR(4) NOT NULL,
    arch CHAR(4) NOT NULL,
    type CHAR(4) NOT NULL,
    key BYTEA NOT NULL,
    PRIMARY KEY (build, platform, arch, type)
);

COMMENT ON TABLE build_auth_key IS 'Keys for authenticating specific builds.';

-- Table: ip_banned
CREATE TABLE ip_banned (
    ip INET NOT NULL DEFAULT '127.0.0.1',
    bandate TIMESTAMP NOT NULL,
    unbandate TIMESTAMP NOT NULL,
    bannedby VARCHAR(50) NOT NULL DEFAULT '[Console]',
    banreason VARCHAR(255) NOT NULL DEFAULT 'No reason',
    PRIMARY KEY (ip, bandate)
);

COMMENT ON TABLE ip_banned IS 'Ban records for IP addresses.';

-- Table: logs
CREATE TABLE logs (
    time BIGINT NOT NULL CHECK (time >= 0),
    realm INT NOT NULL,
    type VARCHAR(250) NOT NULL,
    level SMALLINT NOT NULL DEFAULT 0,
    message TEXT
);

COMMENT ON TABLE logs IS 'Log records for events.';

-- Table: rbac_account_permissions
CREATE TABLE rbac_account_permissions (
    account_id INT NOT NULL,
    permission_id INT NOT NULL,
    granted BOOLEAN NOT NULL DEFAULT TRUE,
    realm_id INT NOT NULL DEFAULT -1,
    PRIMARY KEY (account_id, permission_id, realm_id)
);

COMMENT ON TABLE rbac_account_permissions IS 'Role-based account permissions.';

CREATE TABLE realmcharacters (
    realmid INT NOT NULL CHECK (realmid >= 0),
    acctid INT NOT NULL CHECK (acctid >= 0),
    numchars SMALLINT NOT NULL DEFAULT 0 CHECK (numchars >= 0),
    PRIMARY KEY (realmid, acctid)
);

COMMENT ON TABLE realmcharacters IS 'Tracks the number of characters for each account in each realm.';

-- Table: realmlist
CREATE TABLE realmlist (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    address VARCHAR(255) NOT NULL DEFAULT '127.0.0.1',
    local_address VARCHAR(255) NOT NULL DEFAULT '127.0.0.1',
    local_subnet_mask VARCHAR(255) NOT NULL DEFAULT '255.255.255.0',
    port SMALLINT NOT NULL DEFAULT 8085,
    icon SMALLINT NOT NULL DEFAULT 0,
    flag SMALLINT NOT NULL DEFAULT 2,
    timezone SMALLINT NOT NULL DEFAULT 0,
    allowed_security_level SMALLINT NOT NULL DEFAULT 0,
    population real NOT NULL DEFAULT 0 CHECK (population >= 0),
    gamebuild INT NOT NULL DEFAULT 12340
);

COMMENT ON TABLE realmlist IS 'List of game realms.';

CREATE TABLE uptime (
    realmid INT NOT NULL CHECK (realmid >= 0),
    starttime INT NOT NULL DEFAULT 0 CHECK (starttime >= 0),
    uptime INT NOT NULL DEFAULT 0 CHECK (uptime >= 0),
    maxplayers SMALLINT NOT NULL DEFAULT 0 CHECK (maxplayers >= 0),
    revision VARCHAR(255) NOT NULL DEFAULT 'Trinitycore',
    PRIMARY KEY (realmid, starttime)
);

COMMENT ON TABLE uptime IS 'Uptime system';
