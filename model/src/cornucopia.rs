// This file was generated with `cornucopia`. Do not modify.

#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod types { pub mod public { #[derive( Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)] pub enum Shape { Cuboid,Tetrahedron,Capsule3d,Torus,Cylinder,Cone,ConicalFrustum,Sphere,}impl<'a> postgres_types::ToSql for Shape
{
    fn
    to_sql(&self, ty: &postgres_types::Type, buf: &mut
    postgres_types::private::BytesMut,) -> Result<postgres_types::IsNull,
    Box<dyn std::error::Error + Sync + Send>,>
    {
        let s = match *self { Shape::Cuboid => "Cuboid",Shape::Tetrahedron => "Tetrahedron",Shape::Capsule3d => "Capsule3d",Shape::Torus => "Torus",Shape::Cylinder => "Cylinder",Shape::Cone => "Cone",Shape::ConicalFrustum => "ConicalFrustum",Shape::Sphere => "Sphere",};
        buf.extend_from_slice(s.as_bytes());
        std::result::Result::Ok(postgres_types::IsNull::No)
    } fn accepts(ty: &postgres_types::Type) -> bool
    {
        if ty.name() != "Shape" { return false; } match *ty.kind()
        {
            postgres_types::Kind::Enum(ref variants) =>
            {
                if variants.len() != 8 { return false; }
                variants.iter().all(|v| match &**v
                { "Cuboid" => true,"Tetrahedron" => true,"Capsule3d" => true,"Torus" => true,"Cylinder" => true,"Cone" => true,"ConicalFrustum" => true,"Sphere" => true,_ => false, })
            } _ => false,
        }
    } fn
    to_sql_checked(&self, ty: &postgres_types::Type, out: &mut
    postgres_types::private::BytesMut,) -> Result<postgres_types::IsNull,
    Box<dyn std::error::Error + Sync + Send>>
    { postgres_types::__to_sql_checked(self, ty, out) }
} impl<'a> postgres_types::FromSql<'a> for Shape
{
    fn from_sql(ty: &postgres_types::Type, buf: &'a [u8],) ->
    Result<Shape, Box<dyn std::error::Error + Sync + Send>,>
    {
        match std::str::from_utf8(buf)?
        {
            "Cuboid" => Ok(Shape::Cuboid),"Tetrahedron" => Ok(Shape::Tetrahedron),"Capsule3d" => Ok(Shape::Capsule3d),"Torus" => Ok(Shape::Torus),"Cylinder" => Ok(Shape::Cylinder),"Cone" => Ok(Shape::Cone),"ConicalFrustum" => Ok(Shape::ConicalFrustum),"Sphere" => Ok(Shape::Sphere),s =>
            Result::Err(Into::into(format!("invalid variant `{}`", s))),
        }
    } fn accepts(ty: &postgres_types::Type) -> bool
    {
        if ty.name() != "Shape" { return false; } match *ty.kind()
        {
            postgres_types::Kind::Enum(ref variants) =>
            {
                if variants.len() != 8 { return false; }
                variants.iter().all(|v| match &**v
                { "Cuboid" => true,"Tetrahedron" => true,"Capsule3d" => true,"Torus" => true,"Cylinder" => true,"Cone" => true,"ConicalFrustum" => true,"Sphere" => true,_ => false, })
            } _ => false,
        }
    }
} }}#[allow(clippy::all, clippy::pedantic)] #[allow(unused_variables)]
#[allow(unused_imports)] #[allow(dead_code)] pub mod queries
{ pub mod auth
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive( Debug)] pub struct InsertUserParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,T3: cornucopia_async::StringSql,T4: cornucopia_async::StringSql,> { pub email: T1,pub username: T2,pub password: T3,pub salt: T4,}pub struct I32Query<'a, C: GenericClient, T, const N: usize>
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectUser
{ pub id : i32,pub email : String,pub username : String,pub password : String,pub salt : String,}pub struct SelectUserBorrowed<'a> { pub id : i32,pub email : &'a str,pub username : &'a str,pub password : &'a str,pub salt : &'a str,}
impl<'a> From<SelectUserBorrowed<'a>> for SelectUser
{
    fn from(SelectUserBorrowed { id,email,username,password,salt,}: SelectUserBorrowed<'a>) ->
    Self { Self { id,email: email.into(),username: username.into(),password: password.into(),salt: salt.into(),} }
}pub struct SelectUserQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectUserBorrowed,
    mapper: fn(SelectUserBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectUserQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectUserBorrowed) -> R) ->
    SelectUserQuery<'a,C,R,N>
    {
        SelectUserQuery
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
}pub fn insert_user() -> InsertUserStmt
{ InsertUserStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"User\"
(email, username, \"password\", salt)
VALUES($1, $2, $3, $4)
RETURNING id")) } pub struct
InsertUserStmt(cornucopia_async::private::Stmt); impl InsertUserStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::StringSql,T3:
cornucopia_async::StringSql,T4:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
email: &'a T1,username: &'a T2,password: &'a T3,salt: &'a T4,) -> I32Query<'a,C,
i32, 4>
{
    I32Query
    {
        client, params: [email,username,password,salt,], stmt: &mut self.0, extractor:
        |row| { row.get(0) }, mapper: |it| { it },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,T3: cornucopia_async::StringSql,T4: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertUserParams<T1,T2,T3,T4,>, I32Query<'a, C,
i32, 4>, C> for InsertUserStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertUserParams<T1,T2,T3,T4,>) -> I32Query<'a, C,
    i32, 4>
    { self.bind(client, &params.email,&params.username,&params.password,&params.salt,) }
}pub fn select_user() -> SelectUserStmt
{ SelectUserStmt(cornucopia_async::private::Stmt::new("SELECT id, email, username, password, salt  FROM public.\"User\"
WHERE email=($1)")) } pub struct
SelectUserStmt(cornucopia_async::private::Stmt); impl SelectUserStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
email: &'a T1,) -> SelectUserQuery<'a,C,
SelectUser, 1>
{
    SelectUserQuery
    {
        client, params: [email,], stmt: &mut self.0, extractor:
        |row| { SelectUserBorrowed { id: row.get(0),email: row.get(1),username: row.get(2),password: row.get(3),salt: row.get(4),} }, mapper: |it| { <SelectUser>::from(it) },
    }
} }}pub mod status
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive(Clone,Copy, Debug)] pub struct SelectStatusesParams<> { pub user_id: i32,pub project_id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertStatusParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub color_r: f32,pub color_g: f32,pub color_b: f32,pub project_id: i32,pub shape: super::super::types::public::Shape,}#[derive( Debug)] pub struct UpdateStatusParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub color_r: f32,pub color_g: f32,pub color_b: f32,pub project_id: i32,pub shape: super::super::types::public::Shape,pub id: i32,}#[derive( Debug, Clone, PartialEq,)] pub struct SelectStatuses
{ pub id : i32,pub title : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub color_r : f32,pub color_g : f32,pub color_b : f32,pub project_id : i32,pub shape : super::super::types::public::Shape,}pub struct SelectStatusesBorrowed<'a> { pub id : i32,pub title : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub color_r : f32,pub color_g : f32,pub color_b : f32,pub project_id : i32,pub shape : super::super::types::public::Shape,}
impl<'a> From<SelectStatusesBorrowed<'a>> for SelectStatuses
{
    fn from(SelectStatusesBorrowed { id,title,created_at,updated_at,color_r,color_g,color_b,project_id,shape,}: SelectStatusesBorrowed<'a>) ->
    Self { Self { id,title: title.into(),created_at,updated_at,color_r,color_g,color_b,project_id,shape,} }
}pub struct SelectStatusesQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectStatusesBorrowed,
    mapper: fn(SelectStatusesBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectStatusesQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectStatusesBorrowed) -> R) ->
    SelectStatusesQuery<'a,C,R,N>
    {
        SelectStatusesQuery
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
}pub fn select_statuses() -> SelectStatusesStmt
{ SelectStatusesStmt(cornucopia_async::private::Stmt::new("select distinct  status.id, status.title, status.created_at, status.updated_at, status.color_r, status.color_g, status.color_b, status.project_id, status.shape
FROM public.\"Status\" status
join public.\"ProjectUser\" projectUser on projectUser.project_id = status.project_id 
where projectUser.user_id  = $1 and projectuser.project_id = $2
LIMIT $3
OFFSET $4")) } pub struct
SelectStatusesStmt(cornucopia_async::private::Stmt); impl SelectStatusesStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
user_id: &'a i32,project_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectStatusesQuery<'a,C,
SelectStatuses, 4>
{
    SelectStatusesQuery
    {
        client, params: [user_id,project_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectStatusesBorrowed { id: row.get(0),title: row.get(1),created_at: row.get(2),updated_at: row.get(3),color_r: row.get(4),color_g: row.get(5),color_b: row.get(6),project_id: row.get(7),shape: row.get(8),} }, mapper: |it| { <SelectStatuses>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectStatusesParams<>, SelectStatusesQuery<'a, C,
SelectStatuses, 4>, C> for SelectStatusesStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectStatusesParams<>) -> SelectStatusesQuery<'a, C,
    SelectStatuses, 4>
    { self.bind(client, &params.user_id,&params.project_id,&params.limit,&params.offset,) }
}pub fn insert_status() -> InsertStatusStmt
{ InsertStatusStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"Status\"
(title,  color_r, color_g, color_b, project_id, shape)
VALUES($1, $2, $3, $4, $5, $6)
RETURNING id")) } pub struct
InsertStatusStmt(cornucopia_async::private::Stmt); impl InsertStatusStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,color_r: &'a f32,color_g: &'a f32,color_b: &'a f32,project_id: &'a i32,shape: &'a super::super::types::public::Shape,) -> I32Query<'a,C,
i32, 6>
{
    I32Query
    {
        client, params: [title,color_r,color_g,color_b,project_id,shape,], stmt: &mut self.0, extractor:
        |row| { row.get(0) }, mapper: |it| { it },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertStatusParams<T1,>, I32Query<'a, C,
i32, 6>, C> for InsertStatusStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertStatusParams<T1,>) -> I32Query<'a, C,
    i32, 6>
    { self.bind(client, &params.title,&params.color_r,&params.color_g,&params.color_b,&params.project_id,&params.shape,) }
}pub fn update_status() -> UpdateStatusStmt
{ UpdateStatusStmt(cornucopia_async::private::Stmt::new("UPDATE public.\"Status\"
SET title=$1, color_r=$2, color_g=$3, color_b=$4, project_id=$5, shape=$6
WHERE id=$7")) } pub struct
UpdateStatusStmt(cornucopia_async::private::Stmt); impl UpdateStatusStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,color_r: &'a f32,color_g: &'a f32,color_b: &'a f32,project_id: &'a i32,shape: &'a super::super::types::public::Shape,id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[title,color_r,color_g,color_b,project_id,shape,id,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, UpdateStatusParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for UpdateStatusStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    UpdateStatusParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.title,&params.color_r,&params.color_g,&params.color_b,&params.project_id,&params.shape,&params.id,)) }
}pub fn delete_status() -> DeleteStatusStmt
{ DeleteStatusStmt(cornucopia_async::private::Stmt::new("DELETE FROM public.\"Status\"
WHERE id = $1")) } pub struct
DeleteStatusStmt(cornucopia_async::private::Stmt); impl DeleteStatusStmt
{ pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,]).await
} }}pub mod tags
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive(Clone,Copy, Debug)] pub struct SelectTagsParams<> { pub model_id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertTagParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub model_id: i32,pub position_x: f32,pub position_y: f32,pub position_z: f32,pub created_by_id: i32,}#[derive( Debug)] pub struct UpdateTagParams<T1: cornucopia_async::StringSql,> { pub title: T1,pub position_x: f32,pub position_y: f32,pub position_z: f32,pub status_id: Option<i32>,pub scale_x: f32,pub scale_y: f32,pub scale_z: f32,pub rotation_x: f32,pub rotation_y: f32,pub rotation_z: f32,pub id: i32,}#[derive(Clone,Copy, Debug)] pub struct SelectModelParams<> { pub id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertModelParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> { pub version: i32,pub model_link: T1,pub name: T2,pub project_id: i32,pub created_by_id: i32,}#[derive(Clone,Copy, Debug)] pub struct SelectModelsParams<> { pub project_id: i32,pub user_id: i32,pub limit: i64,pub offset: i64,}#[derive(Clone,Copy, Debug)] pub struct SelectProjectsParams<> { pub user_id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug)] pub struct InsertProjectParams<T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> { pub name: T1,pub description: T2,pub created_by_id: i32,}#[derive( Debug)] pub struct InsertTagMessageParams<T1: cornucopia_async::StringSql,> { pub text: T1,pub tag_id: i32,pub created_by_id: i32,}#[derive(Clone,Copy, Debug)] pub struct SelectTagMessagesParams<> { pub tag_id: i32,pub user_id: i32,pub limit: i64,pub offset: i64,}#[derive( Debug, Clone, PartialEq,)] pub struct Tag
{ pub id : i32,pub email : String,pub username : String,pub title : String,pub model_id : i32,pub position_x : f32,pub position_y : f32,pub position_z : f32,pub created_by_id : i32,pub created_at : time::PrimitiveDateTime,pub shape : Option<super::super::types::public::Shape>,pub scale_x : f32,pub scale_y : f32,pub scale_z : f32,pub rotation_x : f32,pub rotation_y : f32,pub rotation_z : f32,pub status_title : Option<String>,pub status_id : Option<i32>,pub status_color_r : Option<f32>,pub status_color_g : Option<f32>,pub status_color_b : Option<f32>,pub status_project_id : Option<i32>,}pub struct TagBorrowed<'a> { pub id : i32,pub email : &'a str,pub username : &'a str,pub title : &'a str,pub model_id : i32,pub position_x : f32,pub position_y : f32,pub position_z : f32,pub created_by_id : i32,pub created_at : time::PrimitiveDateTime,pub shape : Option<super::super::types::public::Shape>,pub scale_x : f32,pub scale_y : f32,pub scale_z : f32,pub rotation_x : f32,pub rotation_y : f32,pub rotation_z : f32,pub status_title : Option<&'a str>,pub status_id : Option<i32>,pub status_color_r : Option<f32>,pub status_color_g : Option<f32>,pub status_color_b : Option<f32>,pub status_project_id : Option<i32>,}
impl<'a> From<TagBorrowed<'a>> for Tag
{
    fn from(TagBorrowed { id,email,username,title,model_id,position_x,position_y,position_z,created_by_id,created_at,shape,scale_x,scale_y,scale_z,rotation_x,rotation_y,rotation_z,status_title,status_id,status_color_r,status_color_g,status_color_b,status_project_id,}: TagBorrowed<'a>) ->
    Self { Self { id,email: email.into(),username: username.into(),title: title.into(),model_id,position_x,position_y,position_z,created_by_id,created_at,shape,scale_x,scale_y,scale_z,rotation_x,rotation_y,rotation_z,status_title: status_title.map(|v| v.into()),status_id,status_color_r,status_color_g,status_color_b,status_project_id,} }
}pub struct TagQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> TagBorrowed,
    mapper: fn(TagBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> TagQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(TagBorrowed) -> R) ->
    TagQuery<'a,C,R,N>
    {
        TagQuery
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectTagMessages
{ pub id : i32,pub text : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub username : String,pub user_id : i32,pub email : String,pub is_admin : bool,pub created_by_id : i32,}pub struct SelectTagMessagesBorrowed<'a> { pub id : i32,pub text : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub username : &'a str,pub user_id : i32,pub email : &'a str,pub is_admin : bool,pub created_by_id : i32,}
impl<'a> From<SelectTagMessagesBorrowed<'a>> for SelectTagMessages
{
    fn from(SelectTagMessagesBorrowed { id,text,created_at,updated_at,username,user_id,email,is_admin,created_by_id,}: SelectTagMessagesBorrowed<'a>) ->
    Self { Self { id,text: text.into(),created_at,updated_at,username: username.into(),user_id,email: email.into(),is_admin,created_by_id,} }
}pub struct SelectTagMessagesQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectTagMessagesBorrowed,
    mapper: fn(SelectTagMessagesBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectTagMessagesQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectTagMessagesBorrowed) -> R) ->
    SelectTagMessagesQuery<'a,C,R,N>
    {
        SelectTagMessagesQuery
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
{ SelectTagsStmt(cornucopia_async::private::Stmt::new("SELECT DISTINCT tag.id, \"user\".\"email\", \"user\".username, tag.title, tag.model_id, tag.position_x, tag.position_y, tag.position_z, tag.created_by_id, tag.created_at,
\"status\".shape, tag.scale_x, tag.scale_y, tag.scale_z, tag.rotation_x, tag.rotation_y, tag.rotation_z,
\"status\".title as status_title, \"status\".id as status_id, \"status\".color_r as status_color_r, \"status\".color_g as status_color_g, \"status\".color_b as status_color_b, \"status\".project_id as status_project_id
FROM public.\"Tag\" tag
JOIN public.\"User\" \"user\" ON tag.created_by_id = \"user\".id
LEFT JOIN public.\"Status\" \"status\" on tag.status_id = \"status\".id
where tag.model_id = $1
order by tag.created_at DESC 
LIMIT $2
OFFSET $3")) } pub struct
SelectTagsStmt(cornucopia_async::private::Stmt); impl SelectTagsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
model_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> TagQuery<'a,C,
Tag, 3>
{
    TagQuery
    {
        client, params: [model_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { TagBorrowed { id: row.get(0),email: row.get(1),username: row.get(2),title: row.get(3),model_id: row.get(4),position_x: row.get(5),position_y: row.get(6),position_z: row.get(7),created_by_id: row.get(8),created_at: row.get(9),shape: row.get(10),scale_x: row.get(11),scale_y: row.get(12),scale_z: row.get(13),rotation_x: row.get(14),rotation_y: row.get(15),rotation_z: row.get(16),status_title: row.get(17),status_id: row.get(18),status_color_r: row.get(19),status_color_g: row.get(20),status_color_b: row.get(21),status_project_id: row.get(22),} }, mapper: |it| { <Tag>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectTagsParams<>, TagQuery<'a, C,
Tag, 3>, C> for SelectTagsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectTagsParams<>) -> TagQuery<'a, C,
    Tag, 3>
    { self.bind(client, &params.model_id,&params.limit,&params.offset,) }
}pub fn insert_tag() -> InsertTagStmt
{ InsertTagStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"Tag\"
(title, model_id, position_x, position_y, position_z, created_by_id)
VALUES($1, $2, $3, $4, $5, $6)")) } pub struct
InsertTagStmt(cornucopia_async::private::Stmt); impl InsertTagStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,model_id: &'a i32,position_x: &'a f32,position_y: &'a f32,position_z: &'a f32,created_by_id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[title,model_id,position_x,position_y,position_z,created_by_id,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, InsertTagParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for InsertTagStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertTagParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.title,&params.model_id,&params.position_x,&params.position_y,&params.position_z,&params.created_by_id,)) }
}pub fn update_tag() -> UpdateTagStmt
{ UpdateTagStmt(cornucopia_async::private::Stmt::new("UPDATE public.\"Tag\"
SET title=$1, position_x=$2, position_y=$3, position_z=$4, status_id=$5, 
scale_x=$6, scale_y=$7, scale_z=$8, rotation_x=$9, rotation_y=$10, rotation_z=$11
WHERE id=$12")) } pub struct
UpdateTagStmt(cornucopia_async::private::Stmt); impl UpdateTagStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
title: &'a T1,position_x: &'a f32,position_y: &'a f32,position_z: &'a f32,status_id: &'a Option<i32>,scale_x: &'a f32,scale_y: &'a f32,scale_z: &'a f32,rotation_x: &'a f32,rotation_y: &'a f32,rotation_z: &'a f32,id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[title,position_x,position_y,position_z,status_id,scale_x,scale_y,scale_z,rotation_x,rotation_y,rotation_z,id,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, UpdateTagParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for UpdateTagStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    UpdateTagParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.title,&params.position_x,&params.position_y,&params.position_z,&params.status_id,&params.scale_x,&params.scale_y,&params.scale_z,&params.rotation_x,&params.rotation_y,&params.rotation_z,&params.id,)) }
}pub fn delete_tag() -> DeleteTagStmt
{ DeleteTagStmt(cornucopia_async::private::Stmt::new("DELETE FROM public.\"Tag\"
WHERE id = $1")) } pub struct
DeleteTagStmt(cornucopia_async::private::Stmt); impl DeleteTagStmt
{ pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[id,]).await
} }pub fn select_model() -> SelectModelStmt
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
(\"version\", model_link, \"name\", project_id, created_by_id)
VALUES($1, $2, $3, $4, $5)
RETURNING id")) } pub struct
InsertModelStmt(cornucopia_async::private::Stmt); impl InsertModelStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
version: &'a i32,model_link: &'a T1,name: &'a T2,project_id: &'a i32,created_by_id: &'a i32,) -> I32Query<'a,C,
i32, 5>
{
    I32Query
    {
        client, params: [version,model_link,name,project_id,created_by_id,], stmt: &mut self.0, extractor:
        |row| { row.get(0) }, mapper: |it| { it },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertModelParams<T1,T2,>, I32Query<'a, C,
i32, 5>, C> for InsertModelStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertModelParams<T1,T2,>) -> I32Query<'a, C,
    i32, 5>
    { self.bind(client, &params.version,&params.model_link,&params.name,&params.project_id,&params.created_by_id,) }
}pub fn select_models() -> SelectModelsStmt
{ SelectModelsStmt(cornucopia_async::private::Stmt::new("SELECT model.id, model.\"version\", model.model_link, model.\"name\", model.created_at, model.updated_at, model.project_id
FROM public.\"Model\" model
JOIN public.\"ProjectUser\" projectUser ON model.project_id = projectUser.project_id
WHERE model.project_id=($1) AND projectUser.user_id=($2) order by created_at DESC 
LIMIT $3
OFFSET $4")) } pub struct
SelectModelsStmt(cornucopia_async::private::Stmt); impl SelectModelsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
project_id: &'a i32,user_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectModelsQuery<'a,C,
SelectModels, 4>
{
    SelectModelsQuery
    {
        client, params: [project_id,user_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectModelsBorrowed { id: row.get(0),version: row.get(1),model_link: row.get(2),name: row.get(3),created_at: row.get(4),updated_at: row.get(5),project_id: row.get(6),} }, mapper: |it| { <SelectModels>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectModelsParams<>, SelectModelsQuery<'a, C,
SelectModels, 4>, C> for SelectModelsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectModelsParams<>) -> SelectModelsQuery<'a, C,
    SelectModels, 4>
    { self.bind(client, &params.project_id,&params.user_id,&params.limit,&params.offset,) }
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
{ SelectProjectsStmt(cornucopia_async::private::Stmt::new("SELECT DISTINCT project.id, project.\"name\", project.description, project.created_at, project.updated_at
FROM public.\"Project\" project
JOIN public.\"ProjectUser\" projectUser ON project.id = projectUser.project_id
WHERE projectUser.user_id = ($1)
order by created_at DESC
LIMIT $2
OFFSET $3")) } pub struct
SelectProjectsStmt(cornucopia_async::private::Stmt); impl SelectProjectsStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
user_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectProjectsQuery<'a,C,
SelectProjects, 3>
{
    SelectProjectsQuery
    {
        client, params: [user_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectProjectsBorrowed { id: row.get(0),name: row.get(1),description: row.get(2),created_at: row.get(3),updated_at: row.get(4),} }, mapper: |it| { <SelectProjects>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectProjectsParams<>, SelectProjectsQuery<'a, C,
SelectProjects, 3>, C> for SelectProjectsStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectProjectsParams<>) -> SelectProjectsQuery<'a, C,
    SelectProjects, 3>
    { self.bind(client, &params.user_id,&params.limit,&params.offset,) }
}pub fn insert_project() -> InsertProjectStmt
{ InsertProjectStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"Project\" 
(name, description, created_by_id)
VALUES ($1, $2, $3)
RETURNING id")) } pub struct
InsertProjectStmt(cornucopia_async::private::Stmt); impl InsertProjectStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,T2:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
name: &'a T1,description: &'a T2,created_by_id: &'a i32,) -> I32Query<'a,C,
i32, 3>
{
    I32Query
    {
        client, params: [name,description,created_by_id,], stmt: &mut self.0, extractor:
        |row| { row.get(0) }, mapper: |it| { it },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,T2: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
InsertProjectParams<T1,T2,>, I32Query<'a, C,
i32, 3>, C> for InsertProjectStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertProjectParams<T1,T2,>) -> I32Query<'a, C,
    i32, 3>
    { self.bind(client, &params.name,&params.description,&params.created_by_id,) }
}pub fn insert_tag_message() -> InsertTagMessageStmt
{ InsertTagMessageStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"TagMessage\"
(\"text\", tag_id, created_by_id)
VALUES($1, $2, $3)")) } pub struct
InsertTagMessageStmt(cornucopia_async::private::Stmt); impl InsertTagMessageStmt
{ pub async fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
text: &'a T1,tag_id: &'a i32,created_by_id: &'a i32,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[text,tag_id,created_by_id,]).await
} }impl <'a, C: GenericClient + Send + Sync, T1: cornucopia_async::StringSql,>
cornucopia_async::Params<'a, InsertTagMessageParams<T1,>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for InsertTagMessageStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertTagMessageParams<T1,>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.text,&params.tag_id,&params.created_by_id,)) }
}pub fn select_tag_messages() -> SelectTagMessagesStmt
{ SelectTagMessagesStmt(cornucopia_async::private::Stmt::new("SELECT DISTINCT tagMessage.id, tagMessage.\"text\", tagMessage.created_at, tagMessage.updated_at, \"user\".username, \"user\".id as user_id, \"user\".email, projectUser.is_admin, tagMessage.created_by_id
FROM public.\"TagMessage\" tagMessage
JOIN public.\"Tag\" tag ON tagMessage.tag_id = tag.id
JOIN public.\"Model\" model ON tag.model_id = model.id
JOIN public.\"Project\" project ON model.project_id = project.id
JOIN public.\"ProjectUser\" projectUser ON project.id = projectUser.project_id
JOIN public.\"User\" \"user\" ON tagMessage.created_by_id = \"user\".id
WHERE tagMessage.tag_id = ($1) AND projectUser.user_id = ($2)
order by tagMessage.created_at DESC
LIMIT $3
OFFSET $4")) } pub struct
SelectTagMessagesStmt(cornucopia_async::private::Stmt); impl SelectTagMessagesStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
tag_id: &'a i32,user_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectTagMessagesQuery<'a,C,
SelectTagMessages, 4>
{
    SelectTagMessagesQuery
    {
        client, params: [tag_id,user_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectTagMessagesBorrowed { id: row.get(0),text: row.get(1),created_at: row.get(2),updated_at: row.get(3),username: row.get(4),user_id: row.get(5),email: row.get(6),is_admin: row.get(7),created_by_id: row.get(8),} }, mapper: |it| { <SelectTagMessages>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectTagMessagesParams<>, SelectTagMessagesQuery<'a, C,
SelectTagMessages, 4>, C> for SelectTagMessagesStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectTagMessagesParams<>) -> SelectTagMessagesQuery<'a, C,
    SelectTagMessages, 4>
    { self.bind(client, &params.tag_id,&params.user_id,&params.limit,&params.offset,) }
}}pub mod users
{ use futures::{{StreamExt, TryStreamExt}};use futures; use cornucopia_async::GenericClient;#[derive( Debug)] pub struct SelectUsersParams<T1: cornucopia_async::StringSql,> { pub email: T1,pub limit: i64,pub offset: i64,}#[derive(Clone,Copy, Debug)] pub struct SelectUsersInProjectParams<> { pub project_id: i32,pub limit: i64,pub offset: i64,}#[derive(Clone,Copy, Debug)] pub struct InsertProjectUserParams<> { pub project_id: i32,pub user_id: i32,pub is_admin: bool,}#[derive( Debug, Clone, PartialEq,)] pub struct SelectUsers
{ pub id : i32,pub email : String,pub username : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}pub struct SelectUsersBorrowed<'a> { pub id : i32,pub email : &'a str,pub username : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}
impl<'a> From<SelectUsersBorrowed<'a>> for SelectUsers
{
    fn from(SelectUsersBorrowed { id,email,username,created_at,updated_at,}: SelectUsersBorrowed<'a>) ->
    Self { Self { id,email: email.into(),username: username.into(),created_at,updated_at,} }
}pub struct SelectUsersQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectUsersBorrowed,
    mapper: fn(SelectUsersBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectUsersQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectUsersBorrowed) -> R) ->
    SelectUsersQuery<'a,C,R,N>
    {
        SelectUsersQuery
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectUsersInProject
{ pub id : i32,pub email : String,pub username : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub is_admin : bool,}pub struct SelectUsersInProjectBorrowed<'a> { pub id : i32,pub email : &'a str,pub username : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,pub is_admin : bool,}
impl<'a> From<SelectUsersInProjectBorrowed<'a>> for SelectUsersInProject
{
    fn from(SelectUsersInProjectBorrowed { id,email,username,created_at,updated_at,is_admin,}: SelectUsersInProjectBorrowed<'a>) ->
    Self { Self { id,email: email.into(),username: username.into(),created_at,updated_at,is_admin,} }
}pub struct SelectUsersInProjectQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectUsersInProjectBorrowed,
    mapper: fn(SelectUsersInProjectBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectUsersInProjectQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectUsersInProjectBorrowed) -> R) ->
    SelectUsersInProjectQuery<'a,C,R,N>
    {
        SelectUsersInProjectQuery
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
}#[derive( Debug, Clone, PartialEq,)] pub struct SelectUserById
{ pub id : i32,pub email : String,pub username : String,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}pub struct SelectUserByIdBorrowed<'a> { pub id : i32,pub email : &'a str,pub username : &'a str,pub created_at : time::PrimitiveDateTime,pub updated_at : time::PrimitiveDateTime,}
impl<'a> From<SelectUserByIdBorrowed<'a>> for SelectUserById
{
    fn from(SelectUserByIdBorrowed { id,email,username,created_at,updated_at,}: SelectUserByIdBorrowed<'a>) ->
    Self { Self { id,email: email.into(),username: username.into(),created_at,updated_at,} }
}pub struct SelectUserByIdQuery<'a, C: GenericClient, T, const N: usize>
{
    client: &'a  C, params:
    [&'a (dyn postgres_types::ToSql + Sync); N], stmt: &'a mut
    cornucopia_async::private::Stmt, extractor: fn(&tokio_postgres::Row) -> SelectUserByIdBorrowed,
    mapper: fn(SelectUserByIdBorrowed) -> T,
} impl<'a, C, T:'a, const N: usize> SelectUserByIdQuery<'a, C, T, N> where C:
GenericClient
{
    pub fn map<R>(self, mapper: fn(SelectUserByIdBorrowed) -> R) ->
    SelectUserByIdQuery<'a,C,R,N>
    {
        SelectUserByIdQuery
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
}pub fn select_users() -> SelectUsersStmt
{ SelectUsersStmt(cornucopia_async::private::Stmt::new("SELECT id, \"email\", username, created_at, updated_at
FROM public.\"User\"
WHERE email like $1
order by created_at DESC
LIMIT $2
OFFSET $3")) } pub struct
SelectUsersStmt(cornucopia_async::private::Stmt); impl SelectUsersStmt
{ pub fn bind<'a, C:
GenericClient,T1:
cornucopia_async::StringSql,>(&'a mut self, client: &'a  C,
email: &'a T1,limit: &'a i64,offset: &'a i64,) -> SelectUsersQuery<'a,C,
SelectUsers, 3>
{
    SelectUsersQuery
    {
        client, params: [email,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectUsersBorrowed { id: row.get(0),email: row.get(1),username: row.get(2),created_at: row.get(3),updated_at: row.get(4),} }, mapper: |it| { <SelectUsers>::from(it) },
    }
} }impl <'a, C: GenericClient,T1: cornucopia_async::StringSql,> cornucopia_async::Params<'a,
SelectUsersParams<T1,>, SelectUsersQuery<'a, C,
SelectUsers, 3>, C> for SelectUsersStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectUsersParams<T1,>) -> SelectUsersQuery<'a, C,
    SelectUsers, 3>
    { self.bind(client, &params.email,&params.limit,&params.offset,) }
}pub fn select_users_in_project() -> SelectUsersInProjectStmt
{ SelectUsersInProjectStmt(cornucopia_async::private::Stmt::new("SELECT u.id, u.\"email\", u.username, u.created_at, u.updated_at, projectUser.is_admin
FROM public.\"User\" as u
JOIN public.\"ProjectUser\" as projectUser ON projectUser.user_id = u.id
WHERE projectUser.project_id = $1
order by created_at DESC
LIMIT $2
OFFSET $3")) } pub struct
SelectUsersInProjectStmt(cornucopia_async::private::Stmt); impl SelectUsersInProjectStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
project_id: &'a i32,limit: &'a i64,offset: &'a i64,) -> SelectUsersInProjectQuery<'a,C,
SelectUsersInProject, 3>
{
    SelectUsersInProjectQuery
    {
        client, params: [project_id,limit,offset,], stmt: &mut self.0, extractor:
        |row| { SelectUsersInProjectBorrowed { id: row.get(0),email: row.get(1),username: row.get(2),created_at: row.get(3),updated_at: row.get(4),is_admin: row.get(5),} }, mapper: |it| { <SelectUsersInProject>::from(it) },
    }
} }impl <'a, C: GenericClient,> cornucopia_async::Params<'a,
SelectUsersInProjectParams<>, SelectUsersInProjectQuery<'a, C,
SelectUsersInProject, 3>, C> for SelectUsersInProjectStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    SelectUsersInProjectParams<>) -> SelectUsersInProjectQuery<'a, C,
    SelectUsersInProject, 3>
    { self.bind(client, &params.project_id,&params.limit,&params.offset,) }
}pub fn select_user_by_id() -> SelectUserByIdStmt
{ SelectUserByIdStmt(cornucopia_async::private::Stmt::new("SELECT u.id, u.\"email\", u.username, u.created_at, u.updated_at
FROM public.\"User\" AS u
WHERE id = $1")) } pub struct
SelectUserByIdStmt(cornucopia_async::private::Stmt); impl SelectUserByIdStmt
{ pub fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
id: &'a i32,) -> SelectUserByIdQuery<'a,C,
SelectUserById, 1>
{
    SelectUserByIdQuery
    {
        client, params: [id,], stmt: &mut self.0, extractor:
        |row| { SelectUserByIdBorrowed { id: row.get(0),email: row.get(1),username: row.get(2),created_at: row.get(3),updated_at: row.get(4),} }, mapper: |it| { <SelectUserById>::from(it) },
    }
} }pub fn insert_project_user() -> InsertProjectUserStmt
{ InsertProjectUserStmt(cornucopia_async::private::Stmt::new("INSERT INTO public.\"ProjectUser\" 
(project_id, user_id, is_admin)
VALUES ($1, $2, $3)")) } pub struct
InsertProjectUserStmt(cornucopia_async::private::Stmt); impl InsertProjectUserStmt
{ pub async fn bind<'a, C:
GenericClient,>(&'a mut self, client: &'a  C,
project_id: &'a i32,user_id: &'a i32,is_admin: &'a bool,) -> Result<u64, tokio_postgres::Error>
{
    let stmt = self.0.prepare(client).await?;
    client.execute(stmt, &[project_id,user_id,is_admin,]).await
} }impl <'a, C: GenericClient + Send + Sync, >
cornucopia_async::Params<'a, InsertProjectUserParams<>, std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
tokio_postgres::Error>> + Send + 'a>>, C> for InsertProjectUserStmt
{
    fn
    params(&'a mut self, client: &'a  C, params: &'a
    InsertProjectUserParams<>) -> std::pin::Pin<Box<dyn futures::Future<Output = Result<u64,
    tokio_postgres::Error>> + Send + 'a>>
    { Box::pin(self.bind(client, &params.project_id,&params.user_id,&params.is_admin,)) }
}}}