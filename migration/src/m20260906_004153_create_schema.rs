use sea_orm_migration::{prelude::*, schema::*};

#[derive(Iden)]
pub enum User {
    Table,
    id,
    name,
    password,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260906_004153_create_schema"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        _manager.create_table(
            Table::create()
                .table(User::Table)
                .col(
                    ColumnDef::new(User::id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(User::name).string_len(100))
                .col(ColumnDef::new(User::password).string_len(100))
                .to_owned(),
        )
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
    }
}
