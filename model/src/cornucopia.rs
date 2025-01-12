// This file was generated with `cornucopia`. Do not modify.

#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod types { }#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod queries
{ pub mod tags
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive(Clone,Copy, Debug)] pub struct SelectTagsParams<> { pub model_id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertTagParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub model_id: i32,pub position_x: f32,pub position_y: f32,pub position_z: f32,}#[derive( Debug)] pub struct UpdateTagParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub position_x: f32,pub position_y: f32,pub position_z: f32,pub id: i32,}#[derive(Clone,Copy, Debug)] pub struct SelectModelParams<> { pub id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertModelParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> { pub version: i32,pub model_link: T1,pub name: T2,pub project_id: i32,}#[derive(Clone,Copy, Debug)] pub struct SelectModelsParams<> { pub project_id: i32,pub limit: i64,pub offset: i64,}#[derive(Clone,Copy, Debug)] pub struct SelectProjectsParams<> { pub limit: i64,pub offset: i64,}#[derive( Debug, Clone, PartialEq,)] pub struct SelectTags
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectModel
{ pub id : i32,pub version : i32,pub model_link : String,pub name : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub project_id : i32,}pub struct SelectModelBorrowed<'a> { pub id : i32,pub version : i32,pub model_link : &'a str,pub name : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub project_id : i32,}
impl<'a> From<SelectModelBorrowed<'a>> for SelectModel
{
    fn from(SelectModelBorrowed { id,version,model_link,name,created_at,updated_at,project_id,}: SelectModelBorrowed<'a>) ->
    Self { Self { id,version,model_link: model_link.into(),name: name.into(),created_at,updated_at,project_id,} }
}pub struct SelectModelQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectModelBorrowed,
    mapper: fn(SelectModelBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectModelQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectModelBorrowed) -> R) ->
    SelectModelQuery<'a,C,R,N>
    {
        SelectModelQuery
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
}pub struct I32Query<'a, C: GenericClient, T, const N: usize>
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectModels
{ pub id : i32,pub version : i32,pub model_link : String,pub name : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub project_id : i32,}pub struct SelectModelsBorrowed<'a> { pub id : i32,pub version : i32,pub model_link : &'a str,pub name : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub project_id : i32,}
impl<'a> From<SelectModelsBorrowed<'a>> for SelectModels
{
    fn from(SelectModelsBorrowed { id,version,model_link,name,created_at,updated_at,project_id,}: SelectModelsBorrowed<'a>) ->
    Self { Self { id,version,model_link: model_link.into(),name: name.into(),created_at,updated_at,project_id,} }
}pub struct SelectModelsQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectModelsBorrowed,
    mapper: fn(SelectModelsBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectModelsQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectModelsBorrowed) -> R) ->
    SelectModelsQuery<'a,C,R,N>
    {
        SelectModelsQuery
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectProject
{ pub id : i32,pub name : String,pub description : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}pub struct SelectProjectBorrowed<'a> { pub id : i32,pub name : &'a str,pub description : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}
impl<'a> From<SelectProjectBorrowed<'a>> for SelectProject
{
    fn from(SelectProjectBorrowed { id,name,description,created_at,updated_at,}: SelectProjectBorrowed<'a>) ->
    Self { Self { id,name: name.into(),description: description.into(),created_at,updated_at,} }
}pub struct SelectProjectQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectProjectBorrowed,
    mapper: fn(SelectProjectBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectProjectQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectProjectBorrowed) -> R) ->
    SelectProjectQuery<'a,C,R,N>
    {
        SelectProjectQuery
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectProjects
{ pub id : i32,pub name : String,pub description : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}pub struct SelectProjectsBorrowed<'a> { pub id : i32,pub name : &'a str,pub description : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}
impl<'a> From<SelectProjectsBorrowed<'a>> for SelectProjects
{
    fn from(SelectProjectsBorrowed { id,name,description,created_at,updated_at,}: SelectProjectsBorrowed<'a>) ->
    Self { Self { id,name: name.into(),description: description.into(),created_at,updated_at,} }
}pub struct SelectProjectsQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectProjectsBorrowed,
    mapper: fn(SelectProjectsBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectProjectsQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectProjectsBorrowed) -> R) ->
    SelectProjectsQuery<'a,C,R,N>
    {
        SelectProjectsQuery
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
{ SelectTagsStmt(cornucopia_async::private::Stmt::new("SELECT * FROM public.\"Tag\" where model_id = $1 order by created_at DESC 
LIMIT $2
OFFSET $3")) } pub struct
SelectTagsStmt(cornucopia_async::private::Stmt); impl SelectTagsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
model_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectTagsQuery<'a,C,
SelectTags, 3>
{
    SelectTagsQuery
    {
        client, params: [model_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectTagsBorrowed { id: row.get(0),title: row.get(1),created_at: row.get(2),updated_at: row.get(3),position_x: row.get(4),position_y: row.get(5),position_z: row.get(6),model_id: row.get(7),} }, mapper: |it| { <SelectTags>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectTagsParams<>, SelectTagsQuery<'a, C,
SelectTags, 3>, C> for SelectTagsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectTagsParams<>) -> SelectTagsQuery<'a, C,
    SelectTags, 3>
    { self.bind(client, &params.model_id,&params.limit,&params.offset,) }
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
}pub fn update_tag() -> UpdateTagStmt
{ UpdateTagStmt(cornucopia_async::private::Stmt::new("UPDATE public.\"Tag\"
SET title=$1, position_x=$2, position_y=$3, position_z=$4
WHERE id=$5")) } pub struct
UpdateTagStmt(cornucopia_async::private::Stmt); impl UpdateTagStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,position_x: &'a f32,position_y: &'a f32,position_z: &'a f32,id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[title,position_x,position_y,position_z,id,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, UpdateTagParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for UpdateTagStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    UpdateTagParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.title,&params.position_x,&params.position_y,&params.position_z,&params.id,)) }
}pub fn select_model() -> SelectModelStmt
{ SelectModelStmt(cornucopia_async::private::Stmt::new("SELECT id, \"version\", model_link, \"name\", created_at, updated_at, project_id
FROM public.\"Model\" WHERE id=($1) order by created_at DESC 
LIMIT $2
OFFSET $3")) } pub struct
SelectModelStmt(cornucopia_async::private::Stmt); impl SelectModelStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectModelQuery<'a,C,
SelectModel, 3>
{
    SelectModelQuery
    {
        client, params: [id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectModelBorrowed { id: row.get(0),version: row.get(1),model_link: row.get(2),name: row.get(3),created_at: row.get(4),updated_at: row.get(5),project_id: row.get(6),} }, mapper: |it| { <SelectModel>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectModelParams<>, SelectModelQuery<'a, C,
SelectModel, 3>, C> for SelectModelStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectModelParams<>) -> SelectModelQuery<'a, C,
    SelectModel, 3>
    { self.bind(client, &params.id,&params.limit,&params.offset,) }
}pub fn insert_model() -> InsertModelStmt
{ InsertModelStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"Model\"
(\"version\", model_link, \"name\", project_id)
VALUES($1, $2, $3, $4)
RETURNING id")) } pub struct
InsertModelStmt(cornucopia_async::private::Stmt); impl InsertModelStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
version: &'a i32,model_link: &'a T1,name: &'a T2,project_id: &'a i32,) -> I32Query<'a,C,
i32, 4>
{
    I32Query
    {
        client, params: [version,model_link,name,project_id,], stmt: &mut self.0, extractor:
        |row| { row.get(0) }, mapper: |it| { it },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertModelParams<T1,T2,>, I32Query<'a, C,
i32, 4>, C> for InsertModelStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertModelParams<T1,T2,>) -> I32Query<'a, C,
    i32, 4>
    { self.bind(client, &params.version,&params.model_link,&params.name,&params.project_id,) }
}pub fn select_models() -> SelectModelsStmt
{ SelectModelsStmt(cornucopia_async::private::Stmt::new("SELECT id, \"version\", model_link, \"name\", created_at, updated_at, project_id
FROM public.\"Model\"
WHERE project_id=($1) order by created_at DESC 
LIMIT $2
OFFSET $3")) } pub struct
SelectModelsStmt(cornucopia_async::private::Stmt); impl SelectModelsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
project_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectModelsQuery<'a,C,
SelectModels, 3>
{
    SelectModelsQuery
    {
        client, params: [project_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectModelsBorrowed { id: row.get(0),version: row.get(1),model_link: row.get(2),name: row.get(3),created_at: row.get(4),updated_at: row.get(5),project_id: row.get(6),} }, mapper: |it| { <SelectModels>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectModelsParams<>, SelectModelsQuery<'a, C,
SelectModels, 3>, C> for SelectModelsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectModelsParams<>) -> SelectModelsQuery<'a, C,
    SelectModels, 3>
    { self.bind(client, &params.project_id,&params.limit,&params.offset,) }
}pub fn select_project() -> SelectProjectStmt
{ SelectProjectStmt(cornucopia_async::private::Stmt::new("SELECT id, \"name\", description, created_at, updated_at
FROM public.\"Project\"
WHERE id=($1)")) } pub struct
SelectProjectStmt(cornucopia_async::private::Stmt); impl SelectProjectStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> SelectProjectQuery<'a,C,
SelectProject, 1>
{
    SelectProjectQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| { SelectProjectBorrowed { id: row.get(0),name: row.get(1),description: row.get(2),created_at: row.get(3),updated_at: row.get(4),} }, mapper: |it| { <SelectProject>::from(it) },
    }
} }pub fn select_projects() -> SelectProjectsStmt
{ SelectProjectsStmt(cornucopia_async::private::Stmt::new("SELECT id, \"name\", description, created_at, updated_at
FROM public.\"Project\" order by created_at DESC
LIMIT $1
OFFSET $2")) } pub struct
SelectProjectsStmt(cornucopia_async::private::Stmt); impl SelectProjectsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
limit: &'a i64,offset: &'a i64,) -> SelectProjectsQuery<'a,C,
SelectProjects, 2>
{
    SelectProjectsQuery
    {
        client, params: [limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectProjectsBorrowed { id: row.get(0),name: row.get(1),description: row.get(2),created_at: row.get(3),updated_at: row.get(4),} }, mapper: |it| { <SelectProjects>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectProjectsParams<>, SelectProjectsQuery<'a, C,
SelectProjects, 2>, C> for SelectProjectsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectProjectsParams<>) -> SelectProjectsQuery<'a, C,
    SelectProjects, 2>
    { self.bind(client, &params.limit,&params.offset,) }
}}}