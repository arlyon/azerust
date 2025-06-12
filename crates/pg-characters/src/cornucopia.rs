// This file was generated with `cornucopia`. Do not modify.

 #[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod types { } #[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod queries
{  pub mod character
{  #[derive( Debug)] pub struct InsertCharacterParams<T1: cornucopia_async::StringSql,> { pub account: u32,pub guid: u32,pub name: T1,pub race: u8,pub class: u8,pub gender: u8,pub skin: u8,pub face: u8,pub hair_style: u8,pub hair_color: u8,pub facial_style: u8,pub zone: u16,pub map: u16,pub position_x: f32,pub position_y: f32,pub position_z: f32,} #[derive( Debug)] pub struct AddFriendByNameParams<T1: cornucopia_async::StringSql,> { pub id: u32,pub friend_id: u32,pub note: Option<T1>,} #[derive( Debug, Clone, PartialEq,)] pub struct GetCharacterById
{ pub id : u32,pub account : u32,pub name : String,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,} pub struct GetCharacterByIdBorrowed<'a> { pub id : u32,pub account : u32,pub name : &'a str,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,}
impl<'a> From<GetCharacterByIdBorrowed<'a>> for GetCharacterById
{
    fn from(GetCharacterByIdBorrowed { id,account,name,level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,}: GetCharacterByIdBorrowed<'a>) ->
    Self { Self { id,account,name: name.into(),level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct GetCharacterByName
{ pub id : u32,pub account : u32,pub name : String,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,} pub struct GetCharacterByNameBorrowed<'a> { pub id : u32,pub account : u32,pub name : &'a str,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,}
impl<'a> From<GetCharacterByNameBorrowed<'a>> for GetCharacterByName
{
    fn from(GetCharacterByNameBorrowed { id,account,name,level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,}: GetCharacterByNameBorrowed<'a>) ->
    Self { Self { id,account,name: name.into(),level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct GetCharacterByAccount
{ pub id : u32,pub account : u32,pub name : String,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,} pub struct GetCharacterByAccountBorrowed<'a> { pub id : u32,pub account : u32,pub name : &'a str,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,}
impl<'a> From<GetCharacterByAccountBorrowed<'a>> for GetCharacterByAccount
{
    fn from(GetCharacterByAccountBorrowed { id,account,name,level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,}: GetCharacterByAccountBorrowed<'a>) ->
    Self { Self { id,account,name: name.into(),level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct AccountData
{ pub r#type : i16,pub time : u32,pub data : Vec<u8>,} pub struct AccountDataBorrowed<'a> { pub r#type : i16,pub time : u32,pub data : &'a [u8],}
impl<'a> From<AccountDataBorrowed<'a>> for AccountData
{
    fn from(AccountDataBorrowed { r#type,time,data,}: AccountDataBorrowed<'a>) ->
    Self { Self { r#type,time,data: data.into(),} }
} #[derive( Debug, Clone, PartialEq,)] pub struct CharacterAccountData
{ pub r#type : i16,pub time : u32,pub data : Vec<u8>,} pub struct CharacterAccountDataBorrowed<'a> { pub r#type : i16,pub time : u32,pub data : &'a [u8],}
impl<'a> From<CharacterAccountDataBorrowed<'a>> for CharacterAccountData
{
    fn from(CharacterAccountDataBorrowed { r#type,time,data,}: CharacterAccountDataBorrowed<'a>) ->
    Self { Self { r#type,time,data: data.into(),} }
} #[derive( Debug, Clone, PartialEq,)] pub struct ListCharacters
{ pub id : u32,pub account : u32,pub name : String,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,} pub struct ListCharactersBorrowed<'a> { pub id : u32,pub account : u32,pub name : &'a str,pub level : u8,pub race : u8,pub class : u8,pub gender : u8,pub skin_color : u8,pub face : u8,pub hair_style : u8,pub hair_color : u8,pub facial_style : u8,pub zone : u16,pub map : u16,pub position_x : f32,pub position_y : f32,pub position_z : f32,}
impl<'a> From<ListCharactersBorrowed<'a>> for ListCharacters
{
    fn from(ListCharactersBorrowed { id,account,name,level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,}: ListCharactersBorrowed<'a>) ->
    Self { Self { id,account,name: name.into(),level,race,class,gender,skin_color,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,} }
} #[derive( Debug, Clone, PartialEq,)] pub struct GetFriends
{ pub friend : u32,pub flags : u32,pub note : String,} pub struct GetFriendsBorrowed<'a> { pub friend : u32,pub flags : u32,pub note : &'a str,}
impl<'a> From<GetFriendsBorrowed<'a>> for GetFriends
{
    fn from(GetFriendsBorrowed { friend,flags,note,}: GetFriendsBorrowed<'a>) ->
    Self { Self { friend,flags,note: note.into(),} }
}  use futures::{StreamExt, TryStreamExt};use futures; use cornucopia_async::GenericClient; pub struct GetCharacterByIdQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> GetCharacterByIdBorrowed,
    mapper: fn(GetCharacterByIdBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> GetCharacterByIdQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(GetCharacterByIdBorrowed) -> R) ->
    GetCharacterByIdQuery<'a,C,R,N>
    {
        GetCharacterByIdQuery
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
} pub struct GetCharacterByNameQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> GetCharacterByNameBorrowed,
    mapper: fn(GetCharacterByNameBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> GetCharacterByNameQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(GetCharacterByNameBorrowed) -> R) ->
    GetCharacterByNameQuery<'a,C,R,N>
    {
        GetCharacterByNameQuery
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
} pub struct GetCharacterByAccountQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> GetCharacterByAccountBorrowed,
    mapper: fn(GetCharacterByAccountBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> GetCharacterByAccountQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(GetCharacterByAccountBorrowed) -> R) ->
    GetCharacterByAccountQuery<'a,C,R,N>
    {
        GetCharacterByAccountQuery
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
} pub struct I64Query<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> i64,
    mapper: fn(i64) -> T,
} impl<'a, C, T:'a, const N: usize> I64Query<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(i64) -> R) ->
    I64Query<'a,C,R,N>
    {
        I64Query
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
} pub struct BoolQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> bool,
    mapper: fn(bool) -> T,
} impl<'a, C, T:'a, const N: usize> BoolQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(bool) -> R) ->
    BoolQuery<'a,C,R,N>
    {
        BoolQuery
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
} pub struct U32Query<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> u32,
    mapper: fn(u32) -> T,
} impl<'a, C, T:'a, const N: usize> U32Query<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(u32) -> R) ->
    U32Query<'a,C,R,N>
    {
        U32Query
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
} pub struct AccountDataQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> AccountDataBorrowed,
    mapper: fn(AccountDataBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> AccountDataQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(AccountDataBorrowed) -> R) ->
    AccountDataQuery<'a,C,R,N>
    {
        AccountDataQuery
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
} pub struct CharacterAccountDataQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> CharacterAccountDataBorrowed,
    mapper: fn(CharacterAccountDataBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> CharacterAccountDataQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(CharacterAccountDataBorrowed) -> R) ->
    CharacterAccountDataQuery<'a,C,R,N>
    {
        CharacterAccountDataQuery
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
} pub struct ListCharactersQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> ListCharactersBorrowed,
    mapper: fn(ListCharactersBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> ListCharactersQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(ListCharactersBorrowed) -> R) ->
    ListCharactersQuery<'a,C,R,N>
    {
        ListCharactersQuery
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
} pub struct GetFriendsQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> GetFriendsBorrowed,
    mapper: fn(GetFriendsBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> GetFriendsQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(GetFriendsBorrowed) -> R) ->
    GetFriendsQuery<'a,C,R,N>
    {
        GetFriendsQuery
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
} pub fn get_character_by_id() -> GetCharacterByIdStmt
{ GetCharacterByIdStmt(cornucopia_async::private::Stmt::new("SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where guid = $1")) } pub struct
GetCharacterByIdStmt(cornucopia_async::private::Stmt); impl GetCharacterByIdStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
guid: &'a u32,) -> GetCharacterByIdQuery<'a,C,
GetCharacterById, 1>
{
    GetCharacterByIdQuery
    {
        client, params: [guid,], stmt: &mut self.0, extractor:
        |row| {  GetCharacterByIdBorrowed { id: row.get(0),account: row.get(1),name: row.get(2),level: row.get(3),race: row.get(4),class: row.get(5),gender: row.get(6),skin_color: row.get(7),face: row.get(8),hair_style: row.get(9),hair_color: row.get(10),facial_style: row.get(11),zone: row.get(12),map: row.get(13),position_x: row.get(14),position_y: row.get(15),position_z: row.get(16),} }, mapper: |it| { <GetCharacterById>::from(it) },
    }
} } pub fn get_character_by_name() -> GetCharacterByNameStmt
{ GetCharacterByNameStmt(cornucopia_async::private::Stmt::new("SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where name = $1")) } pub struct
GetCharacterByNameStmt(cornucopia_async::private::Stmt); impl GetCharacterByNameStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
name: &'a T1,) -> GetCharacterByNameQuery<'a,C,
GetCharacterByName, 1>
{
    GetCharacterByNameQuery
    {
        client, params: [name,], stmt: &mut self.0, extractor:
        |row| {  GetCharacterByNameBorrowed { id: row.get(0),account: row.get(1),name: row.get(2),level: row.get(3),race: row.get(4),class: row.get(5),gender: row.get(6),skin_color: row.get(7),face: row.get(8),hair_style: row.get(9),hair_color: row.get(10),facial_style: row.get(11),zone: row.get(12),map: row.get(13),position_x: row.get(14),position_y: row.get(15),position_z: row.get(16),} }, mapper: |it| { <GetCharacterByName>::from(it) },
    }
} } pub fn get_character_by_account() -> GetCharacterByAccountStmt
{ GetCharacterByAccountStmt(cornucopia_async::private::Stmt::new("SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters where account = $1")) } pub struct
GetCharacterByAccountStmt(cornucopia_async::private::Stmt); impl GetCharacterByAccountStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
account: &'a u32,) -> GetCharacterByAccountQuery<'a,C,
GetCharacterByAccount, 1>
{
    GetCharacterByAccountQuery
    {
        client, params: [account,], stmt: &mut self.0, extractor:
        |row| {  GetCharacterByAccountBorrowed { id: row.get(0),account: row.get(1),name: row.get(2),level: row.get(3),race: row.get(4),class: row.get(5),gender: row.get(6),skin_color: row.get(7),face: row.get(8),hair_style: row.get(9),hair_color: row.get(10),facial_style: row.get(11),zone: row.get(12),map: row.get(13),position_x: row.get(14),position_y: row.get(15),position_z: row.get(16),} }, mapper: |it| { <GetCharacterByAccount>::from(it) },
    }
} } pub fn count_by_account() -> CountByAccountStmt
{ CountByAccountStmt(cornucopia_async::private::Stmt::new("SELECT count(*) as count FROM characters where account = $1")) } pub struct
CountByAccountStmt(cornucopia_async::private::Stmt); impl CountByAccountStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
account: &'a u32,) -> I64Query<'a,C,
i64, 1>
{
    I64Query
    {
        client, params: [account,], stmt: &mut self.0, extractor:
        |row| {  row.get(0) }, mapper: |it| { it },
    }
} } pub fn name_available() -> NameAvailableStmt
{ NameAvailableStmt(cornucopia_async::private::Stmt::new("SELECT count(*) = 0 as available FROM characters where name = $1")) } pub struct
NameAvailableStmt(cornucopia_async::private::Stmt); impl NameAvailableStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
name: &'a T1,) -> BoolQuery<'a,C,
bool, 1>
{
    BoolQuery
    {
        client, params: [name,], stmt: &mut self.0, extractor:
        |row| {  row.get(0) }, mapper: |it| { it },
    }
} } pub fn insert_character() -> InsertCharacterStmt
{ InsertCharacterStmt(cornucopia_async::private::Stmt::new("INSERT INTO characters
    (account, guid, level, name, race, class, gender, skin, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z, taxi_mask, taxi_path, explored_zones, equipment_cache, known_titles)
values
    ($1, $2, 1, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, '','', '', '', '')
returning
    guid")) } pub struct
InsertCharacterStmt(cornucopia_async::private::Stmt); impl InsertCharacterStmt
{  pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
account: &'a u32,guid: &'a u32,name: &'a T1,race: &'a u8,class: &'a u8,gender: &'a u8,skin: &'a u8,face: &'a u8,hair_style: &'a u8,hair_color: &'a u8,facial_style: &'a u8,zone: &'a u16,map: &'a u16,position_x: &'a f32,position_y: &'a f32,position_z: &'a f32,) -> U32Query<'a,C,
u32, 16>
{
    U32Query
    {
        client, params: [account,guid,name,race,class,gender,skin,face,hair_style,hair_color,facial_style,zone,map,position_x,position_y,position_z,], stmt: &mut self.0, extractor:
        |row| {  row.get(0) }, mapper: |it| { it },
    }
} } impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertCharacterParams<T1,>, U32Query<'a, C,
u32, 16>, C> for InsertCharacterStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertCharacterParams<T1,>) -> U32Query<'a, C,
    u32, 16>
    { self.bind(client, &params.account,&params.guid,&params.name,&params.race,&params.class,&params.gender,&params.skin,&params.face,&params.hair_style,&params.hair_color,&params.facial_style,&params.zone,&params.map,&params.position_x,&params.position_y,&params.position_z,) }
} pub fn delete_character() -> DeleteCharacterStmt
{ DeleteCharacterStmt(cornucopia_async::private::Stmt::new("DELETE FROM characters where guid = $1")) } pub struct
DeleteCharacterStmt(cornucopia_async::private::Stmt); impl DeleteCharacterStmt
{  pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a u32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,]).await
} } pub fn account_data() -> AccountDataStmt
{ AccountDataStmt(cornucopia_async::private::Stmt::new("SELECT type, time, data FROM account_data WHERE account_id = $1")) } pub struct
AccountDataStmt(cornucopia_async::private::Stmt); impl AccountDataStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a u32,) -> AccountDataQuery<'a,C,
AccountData, 1>
{
    AccountDataQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  AccountDataBorrowed { r#type: row.get(0),time: row.get(1),data: row.get(2),} }, mapper: |it| { <AccountData>::from(it) },
    }
} } pub fn character_account_data() -> CharacterAccountDataStmt
{ CharacterAccountDataStmt(cornucopia_async::private::Stmt::new("SELECT type, time, data FROM character_account_data WHERE guid = $1")) } pub struct
CharacterAccountDataStmt(cornucopia_async::private::Stmt); impl CharacterAccountDataStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a u32,) -> CharacterAccountDataQuery<'a,C,
CharacterAccountData, 1>
{
    CharacterAccountDataQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  CharacterAccountDataBorrowed { r#type: row.get(0),time: row.get(1),data: row.get(2),} }, mapper: |it| { <CharacterAccountData>::from(it) },
    }
} } pub fn list_characters() -> ListCharactersStmt
{ ListCharactersStmt(cornucopia_async::private::Stmt::new("SELECT guid as id, account, name, level, race, class, gender, skin as skin_color, face, hair_style, hair_color, facial_style, zone, map, position_x, position_y, position_z FROM characters")) } pub struct
ListCharactersStmt(cornucopia_async::private::Stmt); impl ListCharactersStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
) -> ListCharactersQuery<'a,C,
ListCharacters, 0>
{
    ListCharactersQuery
    {
        client, params: [], stmt: &mut self.0, extractor:
        |row| {  ListCharactersBorrowed { id: row.get(0),account: row.get(1),name: row.get(2),level: row.get(3),race: row.get(4),class: row.get(5),gender: row.get(6),skin_color: row.get(7),face: row.get(8),hair_style: row.get(9),hair_color: row.get(10),facial_style: row.get(11),zone: row.get(12),map: row.get(13),position_x: row.get(14),position_y: row.get(15),position_z: row.get(16),} }, mapper: |it| { <ListCharacters>::from(it) },
    }
} } pub fn get_friends() -> GetFriendsStmt
{ GetFriendsStmt(cornucopia_async::private::Stmt::new("select friend, flags, note from character_social where guid = $1")) } pub struct
GetFriendsStmt(cornucopia_async::private::Stmt); impl GetFriendsStmt
{  pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a u32,) -> GetFriendsQuery<'a,C,
GetFriends, 1>
{
    GetFriendsQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| {  GetFriendsBorrowed { friend: row.get(0),flags: row.get(1),note: row.get(2),} }, mapper: |it| { <GetFriends>::from(it) },
    }
} } pub fn add_friend_by_name() -> AddFriendByNameStmt
{ AddFriendByNameStmt(cornucopia_async::private::Stmt::new("INSERT INTO character_social (guid, friend, note) values ($1, $2, $3)")) } pub struct
AddFriendByNameStmt(cornucopia_async::private::Stmt); impl AddFriendByNameStmt
{  pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
id: &'a u32,friend_id: &'a u32,note: &'a Option<T1>,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,friend_id,note,]).await
} } impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, AddFriendByNameParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for AddFriendByNameStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    AddFriendByNameParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.id,&params.friend_id,&params.note,)) }
} }}