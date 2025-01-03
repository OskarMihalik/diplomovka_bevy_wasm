// This file was generated with `cornucopia`. Do not modify.

#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod types { }#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod queries
{ pub mod tags
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive(Clone,Copy, Debug)] pub struct SelectTagsParams<> { pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertTagParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub model_id: i32,pub position_x: f32,pub position_y: f32,pub position_z: f32,}#[derive( Debug, Clone, PartialEq,)] pub struct SelectTags
{ pub id : i32,pub title : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub position_x : f32,pub position_y : f32,pub position_z : f32,pub model_id : i32,}pub struct SelectTagsBorrowed<'a> { pub id : i32,pub title : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub position_x : f32,pub position_y : f32,pub position_z : f32,pub model_id : i32,}
impl<'a> From<SelectTagsBorrowed<'a>> for SelectTags
{
    fn from(SelectTagsBorrowed { id,title,created_at,updated_at,position_x,position_y,position_z,model_id,}: SelectTagsBorrowed<'a>) ->
    Self { Self { id,title: title.into(),created_at,updated_at,position_x,position_y,position_z,model_id,} }
}pub struct SelectTagsQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectTagsBorrowed,
    mapper: fn(SelectTagsBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectTagsQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectTagsBorrowed) -> R) ->
    SelectTagsQuery<'a,C,R,N>
    {
        SelectTagsQuery
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
}pub fn select_tags() -> SelectTagsStmt
{ SelectTagsStmt(cornucopia_async::private::Stmt::new("SELECT * FROM public.\"Tag\" order by created_at DESC 
LIMIT $1
OFFSET $2")) } pub struct
SelectTagsStmt(cornucopia_async::private::Stmt); impl SelectTagsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
limit: &'a i64,offset: &'a i64,) -> SelectTagsQuery<'a,C,
SelectTags, 2>
{
    SelectTagsQuery
    {
        client, params: [limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectTagsBorrowed { id: row.get(0),title: row.get(1),created_at: row.get(2),updated_at: row.get(3),position_x: row.get(4),position_y: row.get(5),position_z: row.get(6),model_id: row.get(7),} }, mapper: |it| { <SelectTags>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectTagsParams<>, SelectTagsQuery<'a, C,
SelectTags, 2>, C> for SelectTagsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectTagsParams<>) -> SelectTagsQuery<'a, C,
    SelectTags, 2>
    { self.bind(client, &params.limit,&params.offset,) }
}pub fn insert_tag() -> InsertTagStmt
{ InsertTagStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"Tag\"
(title, model_id, position_x, position_y, position_z)
VALUES($1, $2, $3, $4, $5)")) } pub struct
InsertTagStmt(cornucopia_async::private::Stmt); impl InsertTagStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,model_id: &'a i32,position_x: &'a f32,position_y: &'a f32,position_z: &'a f32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[title,model_id,position_x,position_y,position_z,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, InsertTagParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for InsertTagStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertTagParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.title,&params.model_id,&params.position_x,&params.position_y,&params.position_z,)) }
}}}