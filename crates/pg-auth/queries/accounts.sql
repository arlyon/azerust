--! list_account: (last_login?, session_key_auth?)
SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online from account;

--! create_account
INSERT INTO account(username, salt, verifier, reg_mail, email, joindate) VALUES(:username, :salt, :verifier, :reg_main, :email, NOW());

--! insert_realmchars_entry
INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid = account.id WHERE acctid IS NULL;

--! account_exists
SELECT id FROM account WHERE id = :id;

--! select_account_by_id: (session_key_auth?, last_login?)
SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online FROM account WHERE id = :id;

--! select_account_by_username : (last_login?, session_key_auth?)
SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online FROM account WHERE username = :username;

--! select_account_banned
SELECT  (ab.unbandate > now() OR ab.unbandate = ab.bandate) as is_banned, (ab.unbandate = ab.bandate) as is_permabanned FROM account_banned ab where ab.id = :id;

--! initiate_relogin
SELECT a.id, a.username, a.locked, a.lock_country, a.last_ip, a.failed_logins, (ab.unbandate > now() OR ab.unbandate = ab.bandate) as is_banned, (ab.unbandate = ab.bandate) as is_permabanned, aa.security_level, a.session_key_auth as session_key FROM account a LEFT JOIN account_access aa ON a.id = aa.account_id LEFT JOIN account_banned ab ON ab.id = a.id AND ab.active = true WHERE a.username = :username AND a.session_key_auth IS NOT NULL;

--! update_session_information
UPDATE account SET session_key_auth = :session_key, last_ip = :last_ip, last_login = now(), locale = :locale, failed_logins = 0, os = :os WHERE username = :username;

--! ban_user
INSERT INTO account_banned (id, bandate, unbandate, bannedby, banreason) values (:id, now(), :unban_date, :banned_by, :ban_reason);
