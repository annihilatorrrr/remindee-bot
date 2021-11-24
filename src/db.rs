use chrono::{NaiveDateTime, Utc};
use directories::BaseDirs;
use sea_query::{
    ColumnDef, Expr, Iden, Query, SqliteQueryBuilder, Table, Values,
};
use sqlx::{sqlite::SqliteQueryResult, Connection, SqliteConnection};

sea_query::sea_query_driver_sqlite!();
use sea_query_driver_sqlite::{bind_query, bind_query_as};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    Query(sea_query::error::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Database(ref err) => write!(f, "Database error: {}", err),
            Self::Query(ref err) => write!(f, "Query error: {}", err),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err)
    }
}

impl From<sea_query::error::Error> for Error {
    fn from(err: sea_query::error::Error) -> Self {
        Self::Query(err)
    }
}

#[derive(Iden, EnumIter)]
enum CronReminder {
    Table,
    Id,
    UserId,
    CronExpr,
    Time,
    Desc,
    Sent,
    Edit,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CronReminderStruct {
    pub id: u32,
    pub user_id: i64,
    pub cron_expr: String,
    pub time: NaiveDateTime,
    pub desc: String,
    pub sent: bool,
    pub edit: bool,
}

#[derive(Iden, EnumIter)]
enum Reminder {
    Table,
    Id,
    UserId,
    Time,
    Desc,
    Sent,
    Edit,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ReminderStruct {
    pub id: u32,
    pub user_id: i64,
    pub time: NaiveDateTime,
    pub desc: String,
    pub sent: bool,
    pub edit: bool,
}

#[derive(Iden)]
enum UserTimezone {
    Table,
    UserId,
    Timezone,
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserTimezoneStruct {
    pub user_id: i64,
    pub timezone: String,
}

pub async fn get_db_connection() -> Result<SqliteConnection, Error> {
    let base_dirs = BaseDirs::new().unwrap();
    if std::env::consts::OS != "android" {
        SqliteConnection::connect(
            base_dirs
                .data_dir()
                .join("remindee_db.sqlite")
                .to_str()
                .unwrap(),
        )
        .await
        .map_err(From::from)
    } else {
        SqliteConnection::connect("remindee_db.sqlite")
            .await
            .map_err(From::from)
    }
}

pub async fn create_reminder_table() -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let sql = Table::create()
        .table(Reminder::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Reminder::Id)
                .integer()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(Reminder::UserId).integer().not_null())
        .col(ColumnDef::new(Reminder::Time).date_time().not_null())
        .col(ColumnDef::new(Reminder::Desc).text().not_null())
        .col(ColumnDef::new(Reminder::Sent).boolean().not_null())
        .col(ColumnDef::new(Reminder::Edit).boolean().not_null())
        .build(SqliteQueryBuilder);
    sqlx::query(&sql).execute(&mut conn).await?;
    Ok(())
}

async fn execute(
    sql: &str,
    values: &Values,
    conn: &mut SqliteConnection,
) -> Result<SqliteQueryResult, Error> {
    let result = bind_query(sqlx::query(sql), values).execute(conn).await?;
    Ok(result)
}

pub async fn insert_reminder(rem: &ReminderStruct) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::insert()
        .into_table(Reminder::Table)
        .columns(Reminder::iter().skip(2))
        .values(vec![
            rem.user_id.into(),
            rem.time.into(),
            rem.desc.clone().into(),
            rem.sent.into(),
            rem.edit.into(),
        ])?
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn mark_reminder_as_sent(id: u32) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(Reminder::Table)
        .value(Reminder::Sent, true.into())
        .and_where(Expr::col(Reminder::Id).eq(id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn mark_reminder_as_edit(id: u32) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(Reminder::Table)
        .value(Reminder::Edit, true.into())
        .and_where(Expr::col(Reminder::Id).eq(id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn reset_reminders_edit(user_id: i64) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(Reminder::Table)
        .value(Reminder::Edit, false.into())
        .and_where(Expr::col(Reminder::UserId).eq(user_id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn get_edit_reminder(
    user_id: i64,
) -> Result<Option<ReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(Reminder::iter().skip(1))
        .from(Reminder::Table)
        .and_where(Expr::col(Reminder::UserId).eq(user_id))
        .and_where(Expr::col(Reminder::Edit).eq(true))
        .and_where(Expr::col(Reminder::Sent).eq(false))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, ReminderStruct>(&sql), &values)
        .fetch_optional(&mut conn)
        .await
        .map_err(From::from)
}

pub async fn get_active_reminders() -> Result<Vec<ReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(Reminder::iter().skip(1))
        .from(Reminder::Table)
        .and_where(Expr::col(Reminder::Sent).eq(false))
        .and_where(Expr::col(Reminder::Time).lt(Utc::now().naive_utc()))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, ReminderStruct>(&sql), &values)
        .fetch_all(&mut conn)
        .await
        .map_err(From::from)
}

pub async fn get_pending_user_reminders(
    user_id: i64,
) -> Result<Vec<ReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(Reminder::iter().skip(1))
        .from(Reminder::Table)
        .and_where(Expr::col(Reminder::UserId).eq(user_id))
        .and_where(Expr::col(Reminder::Sent).eq(false))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, ReminderStruct>(&sql), &values)
        .fetch_all(&mut conn)
        .await
        .map_err(From::from)
}

pub async fn create_user_timezone_table() -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let sql = Table::create()
        .table(UserTimezone::Table)
        .if_not_exists()
        .col(ColumnDef::new(UserTimezone::UserId).integer().primary_key())
        .col(ColumnDef::new(UserTimezone::Timezone).text().not_null())
        .build(SqliteQueryBuilder);
    sqlx::query(&sql).execute(&mut conn).await?;
    Ok(())
}

pub async fn get_user_timezone_name(
    user_id: i64,
) -> Result<Option<String>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(vec![UserTimezone::UserId, UserTimezone::Timezone])
        .from(UserTimezone::Table)
        .and_where(Expr::col(UserTimezone::UserId).eq(user_id))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, UserTimezoneStruct>(&sql), &values)
        .fetch_optional(&mut conn)
        .await
        .map(|row_opt| row_opt.map(|row| row.timezone))
        .map_err(From::from)
}

async fn update_user_timezone_name(
    conn: &mut SqliteConnection,
    user_id: i64,
    timezone: &str,
) -> Result<(), Error> {
    let (sql, values) = Query::update()
        .table(UserTimezone::Table)
        .value(UserTimezone::Timezone, timezone.to_string().into())
        .and_where(Expr::col(UserTimezone::UserId).eq(user_id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, conn).await?;
    Ok(())
}

async fn insert_user_timezone_name(
    conn: &mut SqliteConnection,
    user_id: i64,
    timezone: &str,
) -> Result<(), Error> {
    let (sql, values) = Query::insert()
        .into_table(UserTimezone::Table)
        .columns(vec![UserTimezone::UserId, UserTimezone::Timezone])
        .values(vec![user_id.into(), timezone.into()])?
        .build(SqliteQueryBuilder);
    execute(&sql, &values, conn).await?;
    Ok(())
}

pub async fn set_user_timezone_name(
    user_id: i64,
    timezone: &str,
) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    match get_user_timezone_name(user_id).await? {
        None => insert_user_timezone_name(&mut conn, user_id, timezone).await?,
        _ => update_user_timezone_name(&mut conn, user_id, timezone).await?,
    }
    Ok(())
}

pub async fn create_cron_reminder_table() -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let sql = Table::create()
        .table(CronReminder::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(CronReminder::Id)
                .integer()
                .primary_key()
                .auto_increment(),
        )
        .col(ColumnDef::new(CronReminder::UserId).integer().not_null())
        .col(ColumnDef::new(CronReminder::CronExpr).text().not_null())
        .col(ColumnDef::new(CronReminder::Time).date_time().not_null())
        .col(ColumnDef::new(CronReminder::Desc).text().not_null())
        .col(ColumnDef::new(CronReminder::Sent).boolean().not_null())
        .col(ColumnDef::new(CronReminder::Edit).boolean().not_null())
        .build(SqliteQueryBuilder);
    sqlx::query(&sql).execute(&mut conn).await?;
    Ok(())
}

pub async fn insert_cron_reminder(
    rem: &CronReminderStruct,
) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::insert()
        .into_table(CronReminder::Table)
        .columns(CronReminder::iter().skip(2))
        .values(vec![
            rem.user_id.into(),
            rem.cron_expr.clone().into(),
            rem.time.into(),
            rem.desc.clone().into(),
            rem.sent.into(),
            rem.edit.into(),
        ])?
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn mark_cron_reminder_as_sent(id: u32) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(CronReminder::Table)
        .value(CronReminder::Sent, true.into())
        .and_where(Expr::col(CronReminder::Id).eq(id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn mark_cron_reminder_as_edit(id: u32) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(CronReminder::Table)
        .value(CronReminder::Edit, true.into())
        .and_where(Expr::col(CronReminder::Id).eq(id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn reset_cron_reminders_edit(user_id: i64) -> Result<(), Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::update()
        .table(CronReminder::Table)
        .value(CronReminder::Edit, false.into())
        .and_where(Expr::col(CronReminder::UserId).eq(user_id))
        .build(SqliteQueryBuilder);
    execute(&sql, &values, &mut conn).await?;
    Ok(())
}

pub async fn get_edit_cron_reminder(
    user_id: i64,
) -> Result<Option<CronReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(CronReminder::iter().skip(1))
        .from(CronReminder::Table)
        .and_where(Expr::col(CronReminder::UserId).eq(user_id))
        .and_where(Expr::col(CronReminder::Edit).eq(true))
        .and_where(Expr::col(CronReminder::Sent).eq(false))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, CronReminderStruct>(&sql), &values)
        .fetch_optional(&mut conn)
        .await
        .map_err(From::from)
}

pub async fn get_active_cron_reminders(
) -> Result<Vec<CronReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(CronReminder::iter().skip(1))
        .from(CronReminder::Table)
        .and_where(Expr::col(CronReminder::Sent).eq(false))
        .and_where(Expr::col(CronReminder::Time).lt(Utc::now().naive_utc()))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, CronReminderStruct>(&sql), &values)
        .fetch_all(&mut conn)
        .await
        .map_err(From::from)
}

pub async fn get_pending_user_cron_reminders(
    user_id: i64,
) -> Result<Vec<CronReminderStruct>, Error> {
    let mut conn = get_db_connection().await?;
    let (sql, values) = Query::select()
        .columns(CronReminder::iter().skip(1))
        .from(CronReminder::Table)
        .and_where(Expr::col(CronReminder::UserId).eq(user_id))
        .and_where(Expr::col(CronReminder::Sent).eq(false))
        .build(SqliteQueryBuilder);
    bind_query_as(sqlx::query_as::<_, CronReminderStruct>(&sql), &values)
        .fetch_all(&mut conn)
        .await
        .map_err(From::from)
}
