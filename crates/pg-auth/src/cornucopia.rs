// This file was generated with `cornucopia`. Do not modify.

 #[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod types { } #[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod queries
{  pub mod accounts
{  #[derive( Debug)] pub struct CreateAccountParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::BytesSql,T3: cornucopia_async::BytesSql,T4: cornucopia_async::StringSql,T5: cornucopia_async::StringSql,> { pub username: T1,pub salt: T2,pub verifier: T3,pub reg_main: T4,pub email: T5,} #[derive( Debug)] pub struct UpdateSessionInformationParams<T1: cornucopia_async::BytesSql,T2: cornucopia_async::StringSql,T3: cornucopia_async::StringSql,> { pub session_key: T1,pub last_ip: std::net::IpAddr,pub locale: u8,pub os: T2,pub username: T3,} #[derive( Debug)] pub struct BanUserParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> { pub id: u32,pub unban_date: time::PrimitiveDateTime,pub banned_by: T1,pub ban_reason: T2,} #[derive( Debug, Clone, PartialEq,)] pub struct ListAccount
{ pub id : i32,pub username : String,pub session_key_auth : Option<Vec<u8>>,pub salt : Vec<u8>,pub verifier : Vec<u8>,pub email : String,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,} pub struct ListAccountBorrowed<'a> { pub id : i32,pub username : &'a str,pub session_key_auth : Option<&'a [u8]>,pub salt : &'a [u8],pub verifier : &'a [u8],pub email : &'a str,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,}
impl<'a> From<ListAccountBorrowed<'a>> for ListAccount
{
    fn from(ListAccountBorrowed { id,username,session_key_auth,salt,verifier,email,joindate,last_login,online,}: ListAccountBorrowed<'a>) ->
    Self { Self { id,username: username.into(),session_key_auth: session_key_auth.map(|v| v.into()),salt: salt.into(),verifier: verifier.into(),email: email.into(),joindate,last_login,online,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct SelectAccountById
{ pub id : i32,pub username : String,pub session_key_auth : Option<Vec<u8>>,pub salt : Vec<u8>,pub verifier : Vec<u8>,pub email : String,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,} pub struct SelectAccountByIdBorrowed<'a> { pub id : i32,pub username : &'a str,pub session_key_auth : Option<&'a [u8]>,pub salt : &'a [u8],pub verifier : &'a [u8],pub email : &'a str,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,}
impl<'a> From<SelectAccountByIdBorrowed<'a>> for SelectAccountById
{
    fn from(SelectAccountByIdBorrowed { id,username,session_key_auth,salt,verifier,email,joindate,last_login,online,}: SelectAccountByIdBorrowed<'a>) ->
    Self { Self { id,username: username.into(),session_key_auth: session_key_auth.map(|v| v.into()),salt: salt.into(),verifier: verifier.into(),email: email.into(),joindate,last_login,online,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct SelectAccountByUsername
{ pub id : i32,pub username : String,pub session_key_auth : Option<Vec<u8>>,pub salt : Vec<u8>,pub verifier : Vec<u8>,pub email : String,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,} pub struct SelectAccountByUsernameBorrowed<'a> { pub id : i32,pub username : &'a str,pub session_key_auth : Option<&'a [u8]>,pub salt : &'a [u8],pub verifier : &'a [u8],pub email : &'a str,pub joindate : time::PrimitiveDateTime,pub last_login : Option<time::PrimitiveDateTime>,pub online : bool,}
impl<'a> From<SelectAccountByUsernameBorrowed<'a>> for SelectAccountByUsername
{
    fn from(SelectAccountByUsernameBorrowed { id,username,session_key_auth,salt,verifier,email,joindate,last_login,online,}: SelectAccountByUsernameBorrowed<'a>) ->
    Self { Self { id,username: username.into(),session_key_auth: session_key_auth.map(|v| v.into()),salt: salt.into(),verifier: verifier.into(),email: email.into(),joindate,last_login,online,} }
} #[derive( Debug, Clone, PartialEq,Copy)] pub struct SelectAccountBanned
{ pub is_banned : bool,pub is_permabanned : bool,} #[derive( Debug, Clone, PartialEq,)] pub struct InitiateRelogin
{ pub id : i32,pub username : String,pub locked : bool,pub lock_country : String,pub last_ip : std::net::IpAddr,pub failed_logins : u32,pub is_banned : bool,pub is_permabanned : bool,pub security_level : u8,pub session_key : Vec<u8>,} pub struct InitiateReloginBorrowed<'a> { pub id : i32,pub username : &'a str,pub locked : bool,pub lock_country : &'a str,pub last_ip : std::net::IpAddr,pub failed_logins : u32,pub is_banned : bool,pub is_permabanned : bool,pub security_level : u8,pub session_key : &'a [u8],}
impl<'a> From<InitiateReloginBorrowed<'a>> for InitiateRelogin
{
    fn from(InitiateReloginBorrowed { id,username,locked,lock_country,last_ip,failed_logins,is_banned,is_permabanned,security_level,session_key,}: InitiateReloginBorrowed<'a>) ->
    Self { Self { id,username: username.into(),locked,lock_country: lock_country.into(),last_ip,failed_logins,is_banned,is_permabanned,security_level,session_key: session_key.into(),} }
}  use futures::{StreamExt, TryStreamExt};use futures; use cornucopia_async::GenericClient; pub struct ListAccountQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> ListAccountBorrowed,
    mapper: fn(ListAccountBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> ListAccountQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(ListAccountBorrowed) -> R) ->
    ListAccountQuery<'a,C,R,N>
    {
        ListAccountQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct I32Query<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> i32,
    mapper: fn(i32) -> T,
} impl<'a, C, T:'a, const N: usize> I32Query<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(i32) -> R) ->
    I32Query<'a,C,R,N>
    {
        I32Query
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct SelectAccountByIdQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectAccountByIdBorrowed,
    mapper: fn(SelectAccountByIdBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectAccountByIdQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectAccountByIdBorrowed) -> R) ->
    SelectAccountByIdQuery<'a,C,R,N>
    {
        SelectAccountByIdQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct SelectAccountByUsernameQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectAccountByUsernameBorrowed,
    mapper: fn(SelectAccountByUsernameBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectAccountByUsernameQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectAccountByUsernameBorrowed) -> R) ->
    SelectAccountByUsernameQuery<'a,C,R,N>
    {
        SelectAccountByUsernameQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct SelectAccountBannedQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectAccountBanned,
    mapper: fn(SelectAccountBanned) -> T,
} impl<'a, C, T:'a, const N: usize> SelectAccountBannedQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectAccountBanned) -> R) ->
    SelectAccountBannedQuery<'a,C,R,N>
    {
        SelectAccountBannedQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct InitiateReloginQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> InitiateReloginBorrowed,
    mapper: fn(InitiateReloginBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> InitiateReloginQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(InitiateReloginBorrowed) -> R) ->
    InitiateReloginQuery<'a,C,R,N>
    {
        InitiateReloginQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub fn list_account() -> ListAccountStmt
{ ListAccountStmt(cornucopia_async::private::Stmt::new("SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online from account")) } pub struct
ListAccountStmt(cornucopia_async::private::Stmt); impl ListAccountStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
) -> ListAccountQuery<'a,C,
ListAccount, 0>
{
    ListAccountQuery
    {
        client, params: [], stmt: &mut self.0, extractor:
        |row| {  ListAccountBorrowed { id: row.get(0),username: row.get(1),session_key_auth: row.get(2),salt: row.get(3),verifier: row.get(4),email: row.get(5),joindate: row.get(6),last_login: row.get(7),online: row.get(8),} }, mapper: |it| { <ListAccount>::from(it) },
    }
} } pub fn create_account() -> CreateAccountStmt
{ CreateAccountStmt(cornucopia_async::private::Stmt::new("INSERT INTO account(username, salt, verifier, reg_mail, email, joindate) VALUES($1, $2, $3, $4, $5, NOW())")) } pub struct
CreateAccountStmt(cornucopia_async::private::Stmt); impl CreateAccountStmt
{  pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::BytesSql,T3:
cornucopia_async::BytesSql,T4:
cornucopia_async::StringSql,T5:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
username: &'a T1,salt: &'a T2,verifier: &'a T3,reg_main: &'a T4,email: &'a T5,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[username,salt,verifier,reg_main,email,]).await
} } impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,T2: cornucopia_async::BytesSql,T3: cornucopia_async::BytesSql,T4: cornucopia_async::StringSql,T5: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, CreateAccountParams<T1,T2,T3,T4,T5,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for CreateAccountStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    CreateAccountParams<T1,T2,T3,T4,T5,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.username,&params.salt,&params.verifier,&params.reg_main,&params.email,)) }
} pub fn insert_realmchars_entry() -> InsertRealmcharsEntryStmt
{ InsertRealmcharsEntryStmt(cornucopia_async::private::Stmt::new("INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT realmlist.id, account.id, 0 FROM realmlist, account LEFT JOIN realmcharacters ON acctid = account.id WHERE acctid IS NULL")) } pub struct
InsertRealmcharsEntryStmt(cornucopia_async::private::Stmt); impl InsertRealmcharsEntryStmt
{  pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[]).await
} } pub fn account_exists() -> AccountExistsStmt
{ AccountExistsStmt(cornucopia_async::private::Stmt::new("SELECT id FROM account WHERE id = $1")) } pub struct
AccountExistsStmt(cornucopia_async::private::Stmt); impl AccountExistsStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> I32Query<'a,C,
i32, 1>
{
    I32Query
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  row.get(0) }, mapper: |it| { it },
    }
} } pub fn select_account_by_id() -> SelectAccountByIdStmt
{ SelectAccountByIdStmt(cornucopia_async::private::Stmt::new("SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online FROM account WHERE id = $1")) } pub struct
SelectAccountByIdStmt(cornucopia_async::private::Stmt); impl SelectAccountByIdStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> SelectAccountByIdQuery<'a,C,
SelectAccountById, 1>
{
    SelectAccountByIdQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  SelectAccountByIdBorrowed { id: row.get(0),username: row.get(1),session_key_auth: row.get(2),salt: row.get(3),verifier: row.get(4),email: row.get(5),joindate: row.get(6),last_login: row.get(7),online: row.get(8),} }, mapper: |it| { <SelectAccountById>::from(it) },
    }
} } pub fn select_account_by_username() -> SelectAccountByUsernameStmt
{ SelectAccountByUsernameStmt(cornucopia_async::private::Stmt::new("SELECT id, username, session_key_auth, salt, verifier, email, joindate, last_login, online FROM account WHERE username = $1")) } pub struct
SelectAccountByUsernameStmt(cornucopia_async::private::Stmt); impl SelectAccountByUsernameStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
username: &'a T1,) -> SelectAccountByUsernameQuery<'a,C,
SelectAccountByUsername, 1>
{
    SelectAccountByUsernameQuery
    {
        client, params: [username,], stmt: &mut self.0, extractor:
        |row| {  SelectAccountByUsernameBorrowed { id: row.get(0),username: row.get(1),session_key_auth: row.get(2),salt: row.get(3),verifier: row.get(4),email: row.get(5),joindate: row.get(6),last_login: row.get(7),online: row.get(8),} }, mapper: |it| { <SelectAccountByUsername>::from(it) },
    }
} } pub fn select_account_banned() -> SelectAccountBannedStmt
{ SelectAccountBannedStmt(cornucopia_async::private::Stmt::new("SELECT  (ab.unbandate > now() OR ab.unbandate = ab.bandate) as is_banned, (ab.unbandate = ab.bandate) as is_permabanned FROM account_banned ab where ab.id = $1")) } pub struct
SelectAccountBannedStmt(cornucopia_async::private::Stmt); impl SelectAccountBannedStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a u32,) -> SelectAccountBannedQuery<'a,C,
SelectAccountBanned, 1>
{
    SelectAccountBannedQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  SelectAccountBanned { is_banned: row.get(0),is_permabanned: row.get(1),} }, mapper: |it| { <SelectAccountBanned>::from(it) },
    }
} } pub fn initiate_relogin() -> InitiateReloginStmt
{ InitiateReloginStmt(cornucopia_async::private::Stmt::new("SELECT a.id, a.username, a.locked, a.lock_country, a.last_ip, a.failed_logins, (ab.unbandate > now() OR ab.unbandate = ab.bandate) as is_banned, (ab.unbandate = ab.bandate) as is_permabanned, aa.security_level, a.session_key_auth as session_key FROM account a LEFT JOIN account_access aa ON a.id = aa.account_id LEFT JOIN account_banned ab ON ab.id = a.id AND ab.active = true WHERE a.username = $1 AND a.session_key_auth IS NOT NULL")) } pub struct
InitiateReloginStmt(cornucopia_async::private::Stmt); impl InitiateReloginStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
username: &'a T1,) -> InitiateReloginQuery<'a,C,
InitiateRelogin, 1>
{
    InitiateReloginQuery
    {
        client, params: [username,], stmt: &mut self.0, extractor:
        |row| {  InitiateReloginBorrowed { id: row.get(0),username: row.get(1),locked: row.get(2),lock_country: row.get(3),last_ip: row.get(4),failed_logins: row.get(5),is_banned: row.get(6),is_permabanned: row.get(7),security_level: row.get(8),session_key: row.get(9),} }, mapper: |it| { <InitiateRelogin>::from(it) },
    }
} } pub fn update_session_information() -> UpdateSessionInformationStmt
{ UpdateSessionInformationStmt(cornucopia_async::private::Stmt::new("UPDATE account SET session_key_auth = $1, last_ip = $2, last_login = now(), locale = $3, failed_logins = 0, os = $4 WHERE username = $5")) } pub struct
UpdateSessionInformationStmt(cornucopia_async::private::Stmt); impl UpdateSessionInformationStmt
{  pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::BytesSql,T2:
cornucopia_async::StringSql,T3:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
session_key: &'a T1,last_ip: &'a std::net::IpAddr,locale: &'a u8,os: &'a T2,username: &'a T3,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[session_key,last_ip,locale,os,username,]).await
} } impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::BytesSql,T2: cornucopia_async::StringSql,T3: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, UpdateSessionInformationParams<T1,T2,T3,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for UpdateSessionInformationStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    UpdateSessionInformationParams<T1,T2,T3,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.session_key,&params.last_ip,&params.locale,&params.os,&params.username,)) }
} pub fn ban_user() -> BanUserStmt
{ BanUserStmt(cornucopia_async::private::Stmt::new("INSERT INTO account_banned (id, bandate, unbandate, bannedby, banreason) values ($1, now(), $2, $3, $4)")) } pub struct
BanUserStmt(cornucopia_async::private::Stmt); impl BanUserStmt
{  pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
id: &'a u32,unban_date: &'a time::PrimitiveDateTime,banned_by: &'a T1,ban_reason: &'a T2,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,unban_date,banned_by,ban_reason,]).await
} } impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, BanUserParams<T1,T2,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for BanUserStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    BanUserParams<T1,T2,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.id,&params.unban_date,&params.banned_by,&params.ban_reason,)) }
} } pub mod realms
{  #[derive(Clone,Copy, Debug)] pub struct UpdateRealmlistStatusParams<> { pub flag: u8,pub id: i32,} #[derive( Debug)] pub struct CreateRealmParams<T1: cornucopia_async::StringSql,> { pub name: T1,pub address: std::net::IpAddr,pub local_address: std::net::IpAddr,pub local_subnet_mask: std::net::IpAddr,pub port: u16,} #[derive( Debug)] pub struct SetUptimeParams<T1: cornucopia_async::StringSql,> { pub id: u32,pub start: time::PrimitiveDateTime,pub uptime: u32,pub maxplayers: u16,pub version: T1,} #[derive( Debug, Clone, PartialEq,)] pub struct RefreshRealmlist
{ pub id : i32,pub name : String,pub realm_type : u8,pub build : u32,pub external_address : std::net::IpAddr,pub local_address : std::net::IpAddr,pub local_subnet_mask : std::net::IpAddr,pub port : u16,pub flags : u8,pub timezone : u8,pub population : f32,} pub struct RefreshRealmlistBorrowed<'a> { pub id : i32,pub name : &'a str,pub realm_type : u8,pub build : u32,pub external_address : std::net::IpAddr,pub local_address : std::net::IpAddr,pub local_subnet_mask : std::net::IpAddr,pub port : u16,pub flags : u8,pub timezone : u8,pub population : f32,}
impl<'a> From<RefreshRealmlistBorrowed<'a>> for RefreshRealmlist
{
    fn from(RefreshRealmlistBorrowed { id,name,realm_type,build,external_address,local_address,local_subnet_mask,port,flags,timezone,population,}: RefreshRealmlistBorrowed<'a>) ->
    Self { Self { id,name: name.into(),realm_type,build,external_address,local_address,local_subnet_mask,port,flags,timezone,population,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct CreateRealm
{ pub id : i32,pub name : String,pub address : std::net::IpAddr,pub local_address : std::net::IpAddr,pub local_subnet_mask : std::net::IpAddr,pub port : u16,pub icon : u8,pub flag : u8,pub timezone : u8,pub allowed_security_level : u8,pub population : f32,pub gamebuild : u32,} pub struct CreateRealmBorrowed<'a> { pub id : i32,pub name : &'a str,pub address : std::net::IpAddr,pub local_address : std::net::IpAddr,pub local_subnet_mask : std::net::IpAddr,pub port : u16,pub icon : u8,pub flag : u8,pub timezone : u8,pub allowed_security_level : u8,pub population : f32,pub gamebuild : u32,}
impl<'a> From<CreateRealmBorrowed<'a>> for CreateRealm
{
    fn from(CreateRealmBorrowed { id,name,address,local_address,local_subnet_mask,port,icon,flag,timezone,allowed_security_level,population,gamebuild,}: CreateRealmBorrowed<'a>) ->
    Self { Self { id,name: name.into(),address,local_address,local_subnet_mask,port,icon,flag,timezone,allowed_security_level,population,gamebuild,} }
}  use futures::{StreamExt, TryStreamExt};use futures; use cornucopia_async::GenericClient; pub struct RefreshRealmlistQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> RefreshRealmlistBorrowed,
    mapper: fn(RefreshRealmlistBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> RefreshRealmlistQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(RefreshRealmlistBorrowed) -> R) ->
    RefreshRealmlistQuery<'a,C,R,N>
    {
        RefreshRealmlistQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub struct CreateRealmQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> CreateRealmBorrowed,
    mapper: fn(CreateRealmBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> CreateRealmQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(CreateRealmBorrowed) -> R) ->
    CreateRealmQuery<'a,C,R,N>
    {
        CreateRealmQuery
        {
            client: self.client, params: self.params, stmt: self.stmt,
            extractor: self.extractor, mapper,
        }
    } pub async fn one(self) -> Result<T, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let row =
        self.client.query_one(stmt, &self.params).await?;
        Ok((self.mapper)((self.extractor)(&row)))
    } pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error>
    { self.iter().await?.try_collect().await } pub async fn opt(self) ->
    Result<Option<T>, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?;
        Ok(self.client.query_opt(stmt, &self.params) .await?
        .map(|row| (self.mapper)((self.extractor)(&row))))
    } pub async fn iter(self,) -> Result<impl futures::Stream<Item = Result<T,
    tokio_postgres::Error>> + 'a, tokio_postgres::Error>
    {
        let stmt = self.stmt.prepare(self.client).await?; let it =
        self.client.query_raw(stmt,
        cornucopia_async::private::slice_iter(&self.params)) .await?
        .map(move |res|
        res.map(|row| (self.mapper)((self.extractor)(&row)))) .into_stream();
        Ok(it)
    }
} pub fn refresh_realmlist() -> RefreshRealmlistStmt
{ RefreshRealmlistStmt(cornucopia_async::private::Stmt::new("SELECT id, name, icon as realm_type, gamebuild as build, address as external_address, local_address, local_subnet_mask, port, flag as flags, timezone, population FROM realmlist WHERE flag <> 3 ORDER BY id")) } pub struct
RefreshRealmlistStmt(cornucopia_async::private::Stmt); impl RefreshRealmlistStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
) -> RefreshRealmlistQuery<'a,C,
RefreshRealmlist, 0>
{
    RefreshRealmlistQuery
    {
        client, params: [], stmt: &mut self.0, extractor:
        |row| {  RefreshRealmlistBorrowed { id: row.get(0),name: row.get(1),realm_type: row.get(2),build: row.get(3),external_address: row.get(4),local_address: row.get(5),local_subnet_mask: row.get(6),port: row.get(7),flags: row.get(8),timezone: row.get(9),population: row.get(10),} }, mapper: |it| { <RefreshRealmlist>::from(it) },
    }
} } pub fn update_realmlist_status() -> UpdateRealmlistStatusStmt
{ UpdateRealmlistStatusStmt(cornucopia_async::private::Stmt::new("update realmlist set flag = $1 where id = $2")) } pub struct
UpdateRealmlistStatusStmt(cornucopia_async::private::Stmt); impl UpdateRealmlistStatusStmt
{  pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
flag: &'a u8,id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[flag,id,]).await
} } impl <'a, C: GenericClient + Send + Sync, >
cornucopia_async::Params<'a, UpdateRealmlistStatusParams<>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for UpdateRealmlistStatusStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    UpdateRealmlistStatusParams<>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.flag,&params.id,)) }
} pub fn create_realm() -> CreateRealmStmt
{ CreateRealmStmt(cornucopia_async::private::Stmt::new("insert into realmlist (name, address, local_address, local_subnet_mask, port) values ($1, $2, $3, $4, $5) returning *")) } pub struct
CreateRealmStmt(cornucopia_async::private::Stmt); impl CreateRealmStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
name: &'a T1,address: &'a std::net::IpAddr,local_address: &'a std::net::IpAddr,local_subnet_mask: &'a std::net::IpAddr,port: &'a u16,) -> CreateRealmQuery<'a,C,
CreateRealm, 5>
{
    CreateRealmQuery
    {
        client, params: [name,address,local_address,local_subnet_mask,port,], stmt: &mut self.0, extractor:
        |row| {  CreateRealmBorrowed { id: row.get(0),name: row.get(1),address: row.get(2),local_address: row.get(3),local_subnet_mask: row.get(4),port: row.get(5),icon: row.get(6),flag: row.get(7),timezone: row.get(8),allowed_security_level: row.get(9),population: row.get(10),gamebuild: row.get(11),} }, mapper: |it| { <CreateRealm>::from(it) },
    }
} } impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
CreateRealmParams<T1,>, CreateRealmQuery<'a, C,
CreateRealm, 5>, C> for CreateRealmStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    CreateRealmParams<T1,>) -> CreateRealmQuery<'a, C,
    CreateRealm, 5>
    { self.bind(client, &params.name,&params.address,&params.local_address,&params.local_subnet_mask,&params.port,) }
} pub fn set_uptime() -> SetUptimeStmt
{ SetUptimeStmt(cornucopia_async::private::Stmt::new("INSERT INTO uptime (realmid, starttime, uptime, maxplayers, revision)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (realmid)
DO UPDATE SET
    uptime = EXCLUDED.uptime,
    maxplayers = EXCLUDED.maxplayers")) } pub struct
SetUptimeStmt(cornucopia_async::private::Stmt); impl SetUptimeStmt
{  pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
id: &'a u32,start: &'a time::PrimitiveDateTime,uptime: &'a u32,maxplayers: &'a u16,version: &'a T1,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,start,uptime,maxplayers,version,]).await
} } impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, SetUptimeParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for SetUptimeStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SetUptimeParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.id,&params.start,&params.uptime,&params.maxplayers,&params.version,)) }
} }}