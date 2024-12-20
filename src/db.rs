use std::fs::OpenOptions;
use std::path::PathBuf;

use crate::entity::common::EditMode;
use crate::entity::{cron_reminder, reminder, user_timezone};
use crate::generic_reminder;
use crate::migration::{DbErr, Migrator, MigratorTrait};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database as SeaOrmDatabase,
    DatabaseConnection, EntityTrait, QueryFilter, Set,
};

#[derive(Debug)]
pub enum Error {
    Database(DbErr),
    File(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Database(ref err) => {
                write!(f, "Database error: {}", err)
            }
            Self::File(ref err) => write!(f, "File error: {}", err),
        }
    }
}

impl From<DbErr> for Error {
    fn from(err: DbErr) -> Self {
        Self::Database(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::File(err)
    }
}

async fn get_db_pool(db_path: &PathBuf) -> Result<DatabaseConnection, Error> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(db_path)?;
    let db_str = format!("sqlite:{}", db_path.display());
    let pool = SeaOrmDatabase::connect(&db_str).await?;
    Ok(pool)
}

#[derive(Clone)]
pub struct Database {
    pool: DatabaseConnection,
}

impl Database {
    pub async fn new(db_path: &PathBuf) -> Result<Self, Error> {
        get_db_pool(db_path).await.map(|pool| Self { pool })
    }

    pub async fn apply_migrations(&self) -> Result<(), Error> {
        Ok(Migrator::up(&self.pool, None).await?)
    }

    pub async fn get_reminder(
        &self,
        id: i64,
    ) -> Result<Option<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::Id.eq(id))
            .one(&self.pool)
            .await?)
    }

    pub async fn insert_reminder(
        &self,
        rem: reminder::ActiveModel,
    ) -> Result<reminder::ActiveModel, Error> {
        Ok(rem.save(&self.pool).await?)
    }

    pub async fn delete_reminder(&self, id: i64) -> Result<(), Error> {
        reminder::ActiveModel {
            id: Set(id),
            ..Default::default()
        }
        .delete(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_reminder_as_edit(
        &self,
        rem_id: i64,
        chat_id: i64,
    ) -> Result<(), Error> {
        self.reset_reminders_edit(chat_id).await?;
        if let Some(mut rem_act) = reminder::Entity::find_by_id(rem_id)
            .one(&self.pool)
            .await?
            .map(Into::<reminder::ActiveModel>::into)
        {
            rem_act.edit = Set(true);
            rem_act.update(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn reset_reminders_edit(
        &self,
        chat_id: i64,
    ) -> Result<(), Error> {
        reminder::Entity::update_many()
            .filter(reminder::Column::ChatId.eq(chat_id))
            .set(reminder::ActiveModel {
                edit: Set(false),
                edit_mode: Set(EditMode::None),
                ..Default::default()
            })
            .exec(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_edit_reminder(
        &self,
        chat_id: i64,
    ) -> Result<Option<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::ChatId.eq(chat_id))
            .filter(reminder::Column::Edit.eq(true))
            .one(&self.pool)
            .await?)
    }

    pub async fn set_edit_mode_reminder(
        &self,
        chat_id: i64,
        edit_mode: EditMode,
    ) -> Result<(), Error> {
        if let Some(mut reminder) = self
            .get_edit_reminder(chat_id)
            .await?
            .map(Into::<reminder::ActiveModel>::into)
        {
            reminder.edit_mode = Set(edit_mode);
            reminder.update(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn set_edit_mode_cron_reminder(
        &self,
        chat_id: i64,
        edit_mode: EditMode,
    ) -> Result<(), Error> {
        if let Some(mut cron_reminder) = self
            .get_edit_cron_reminder(chat_id)
            .await?
            .map(Into::<cron_reminder::ActiveModel>::into)
        {
            cron_reminder.edit_mode = Set(edit_mode);
            cron_reminder.update(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn update_edited_reminder_description(
        &self,
        rem: reminder::Model,
    ) -> Result<(), Error> {
        let desc = rem.desc.clone();
        let mut rem_act = Into::<reminder::ActiveModel>::into(rem);
        rem_act.edit = Set(false);
        rem_act.edit_mode = Set(EditMode::None);
        rem_act.desc = Set(desc);
        rem_act.update(&self.pool).await?;
        Ok(())
    }

    pub async fn get_active_reminders(
        &self,
    ) -> Result<Vec<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::Paused.eq(false))
            .filter(reminder::Column::Time.lt(Utc::now().naive_utc()))
            .all(&self.pool)
            .await?)
    }

    pub async fn get_pending_chat_reminders(
        &self,
        chat_id: i64,
    ) -> Result<Vec<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::ChatId.eq(chat_id))
            .all(&self.pool)
            .await?)
    }

    pub async fn get_user_timezone_name(
        &self,
        user_id: i64,
    ) -> Result<Option<String>, Error> {
        Ok(user_timezone::Entity::find_by_id(user_id)
            .one(&self.pool)
            .await?
            .map(|x| x.timezone))
    }

    async fn insert_user_timezone_name(
        &self,
        user_id: i64,
        timezone: &str,
    ) -> Result<(), Error> {
        user_timezone::Entity::insert(user_timezone::ActiveModel {
            user_id: Set(user_id),
            timezone: Set(timezone.to_string()),
        })
        .exec(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_or_update_user_timezone(
        &self,
        user_id: i64,
        timezone: &str,
    ) -> Result<(), Error> {
        if let Some(mut tz_act) = user_timezone::Entity::find_by_id(user_id)
            .one(&self.pool)
            .await?
            .map(Into::<user_timezone::ActiveModel>::into)
        {
            tz_act.timezone = Set(timezone.to_string());
            tz_act.update(&self.pool).await?;
        } else {
            self.insert_user_timezone_name(user_id, timezone).await?;
        }
        Ok(())
    }

    pub async fn get_cron_reminder(
        &self,
        id: i64,
    ) -> Result<Option<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::Id.eq(id))
            .one(&self.pool)
            .await?)
    }

    pub async fn insert_cron_reminder(
        &self,
        rem: cron_reminder::ActiveModel,
    ) -> Result<cron_reminder::ActiveModel, Error> {
        Ok(rem.save(&self.pool).await?)
    }

    pub async fn delete_cron_reminder(&self, id: i64) -> Result<(), Error> {
        cron_reminder::ActiveModel {
            id: Set(id),
            ..Default::default()
        }
        .delete(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_cron_reminder_as_edit(
        &self,
        rem_id: i64,
        chat_id: i64,
    ) -> Result<(), Error> {
        self.reset_cron_reminders_edit(chat_id).await?;
        if let Some(mut cron_rem_act) =
            cron_reminder::Entity::find_by_id(rem_id)
                .one(&self.pool)
                .await?
                .map(Into::<cron_reminder::ActiveModel>::into)
        {
            cron_rem_act.edit = Set(true);
            cron_rem_act.update(&self.pool).await?;
        }
        Ok(())
    }

    pub async fn reset_cron_reminders_edit(
        &self,
        chat_id: i64,
    ) -> Result<(), Error> {
        cron_reminder::Entity::update_many()
            .filter(cron_reminder::Column::ChatId.eq(chat_id))
            .set(cron_reminder::ActiveModel {
                edit: Set(false),
                edit_mode: Set(EditMode::None),
                ..Default::default()
            })
            .exec(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_edit_cron_reminder(
        &self,
        chat_id: i64,
    ) -> Result<Option<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::ChatId.eq(chat_id))
            .filter(cron_reminder::Column::Edit.eq(true))
            .one(&self.pool)
            .await?)
    }

    pub async fn toggle_reminder_paused(&self, id: i64) -> Result<bool, Error> {
        let rem: Option<reminder::Model> =
            reminder::Entity::find_by_id(id).one(&self.pool).await?;
        if let Some(rem) = rem {
            let paused_value = !rem.paused;
            let mut rem_act: reminder::ActiveModel = rem.into();
            rem_act.paused = Set(paused_value);
            rem_act.update(&self.pool).await?;
            Ok(paused_value)
        } else {
            Err(Error::Database(DbErr::RecordNotFound(id.to_string())))
        }
    }

    pub async fn toggle_cron_reminder_paused(
        &self,
        id: i64,
    ) -> Result<bool, Error> {
        let cron_rem: Option<cron_reminder::Model> =
            cron_reminder::Entity::find_by_id(id)
                .one(&self.pool)
                .await?;
        if let Some(cron_rem) = cron_rem {
            let paused_value = !cron_rem.paused;
            let mut cron_rem_act: cron_reminder::ActiveModel = cron_rem.into();
            cron_rem_act.paused = Set(paused_value);
            cron_rem_act.update(&self.pool).await?;
            Ok(paused_value)
        } else {
            Err(Error::Database(DbErr::RecordNotFound(id.to_string())))
        }
    }

    pub async fn get_active_cron_reminders(
        &self,
    ) -> Result<Vec<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::Paused.eq(false))
            .filter(cron_reminder::Column::Time.lt(Utc::now().naive_utc()))
            .all(&self.pool)
            .await?)
    }

    pub async fn get_pending_chat_cron_reminders(
        &self,
        chat_id: i64,
    ) -> Result<Vec<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::ChatId.eq(chat_id))
            .all(&self.pool)
            .await?)
    }

    pub async fn get_sorted_reminders(
        &self,
        chat_id: i64,
        exclude_reminders: bool,
        exclude_cron_reminders: bool,
    ) -> Result<Vec<Box<dyn generic_reminder::GenericReminder>>, Error> {
        let reminders = self
            .get_pending_chat_reminders(chat_id)
            .await?
            .into_iter()
            .map(|x| -> Box<dyn generic_reminder::GenericReminder> {
                Box::<reminder::ActiveModel>::new(x.into())
            });
        let cron_reminders = self
            .get_pending_chat_cron_reminders(chat_id)
            .await?
            .into_iter()
            .map(|x| -> Box<dyn generic_reminder::GenericReminder> {
                Box::<cron_reminder::ActiveModel>::new(x.into())
            });

        let mut all_reminders = vec![];
        if !exclude_reminders {
            all_reminders.extend(reminders)
        }
        if !exclude_cron_reminders {
            all_reminders.extend(cron_reminders)
        }
        all_reminders.sort_unstable();
        Ok(all_reminders)
    }

    pub async fn get_sorted_all_reminders(
        &self,
        chat_id: i64,
    ) -> Result<Vec<Box<dyn generic_reminder::GenericReminder>>, Error> {
        self.get_sorted_reminders(chat_id, false, false).await
    }

    pub async fn get_reminder_by_msg_id(
        &self,
        msg_id: i32,
    ) -> Result<Option<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::MsgId.eq(msg_id))
            .one(&self.pool)
            .await?)
    }

    pub async fn get_cron_reminder_by_msg_id(
        &self,
        msg_id: i32,
    ) -> Result<Option<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::MsgId.eq(msg_id))
            .one(&self.pool)
            .await?)
    }

    pub async fn get_reminder_by_reply_id(
        &self,
        reply_id: i32,
    ) -> Result<Option<reminder::Model>, Error> {
        Ok(reminder::Entity::find()
            .filter(reminder::Column::ReplyId.eq(reply_id))
            .one(&self.pool)
            .await?)
    }

    pub async fn get_cron_reminder_by_reply_id(
        &self,
        reply_id: i32,
    ) -> Result<Option<cron_reminder::Model>, Error> {
        Ok(cron_reminder::Entity::find()
            .filter(cron_reminder::Column::ReplyId.eq(reply_id))
            .one(&self.pool)
            .await?)
    }

    pub async fn set_reminder_reply_id(
        &self,
        mut rem: reminder::ActiveModel,
        reply_id: i32,
    ) -> Result<(), Error> {
        rem.reply_id = Set(Some(reply_id));
        rem.update(&self.pool).await?;
        Ok(())
    }

    pub async fn set_cron_reminder_reply_id(
        &self,
        mut cron_rem: cron_reminder::ActiveModel,
        reply_id: i32,
    ) -> Result<(), Error> {
        cron_rem.reply_id = Set(Some(reply_id));
        cron_rem.update(&self.pool).await?;
        Ok(())
    }
}
