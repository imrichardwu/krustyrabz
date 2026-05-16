use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create UserAccount table
        manager
            .create_table(
                Table::create()
                    .table(UserAccount::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserAccount::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserAccount::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(UserAccount::TokenBalance).double())
                    .col(ColumnDef::new(UserAccount::RoundsPlayed).integer())
                    .col(ColumnDef::new(UserAccount::PotsWon).integer())
                    .col(ColumnDef::new(UserAccount::NumberFolds).integer())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop UserAccount table
        manager
            .drop_table(Table::drop().table(UserAccount::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserAccount {
    Table,
    Id,
    Username,
    TokenBalance,
    RoundsPlayed,
    PotsWon,
    NumberFolds,
}
