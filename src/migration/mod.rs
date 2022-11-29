pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_user_timezone_table;
pub mod m20221111_004928_create_reminder_table;
pub mod m20221111_005303_create_cron_reminder_table;
pub mod m20221113_214952_create_user_id_columns;
pub mod m20221115_001608_set_user_id_to_chat_id;
mod m20221119_222755_create_paused_columns;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_user_timezone_table::Migration),
            Box::new(m20221111_004928_create_reminder_table::Migration),
            Box::new(m20221111_005303_create_cron_reminder_table::Migration),
            Box::new(m20221113_214952_create_user_id_columns::Migration),
            Box::new(m20221115_001608_set_user_id_to_chat_id::Migration),
            Box::new(m20221119_222755_create_paused_columns::Migration),
        ]
    }
}
