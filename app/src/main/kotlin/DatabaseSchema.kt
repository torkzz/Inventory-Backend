package com.kevinfreyap

import data.table.user.GroupTable
import data.table.auth.RefreshTokenTable
import data.table.user.UserGroupTable
import data.table.user.UserTable
import data.table.product.CategoryTable
import data.table.product.DeletedTable
import data.table.product.LocationTable
import data.table.product.ProductTable
import data.table.product.StockBatchTable
import data.table.user.GroupInviteTable
import org.jetbrains.exposed.v1.jdbc.SchemaUtils
import org.jetbrains.exposed.v1.jdbc.transactions.transaction

fun initializeDatabaseSchema() {
    transaction {
        SchemaUtils.create(
            CategoryTable,
            LocationTable,
            ProductTable,
            StockBatchTable,
            DeletedTable,

            UserTable,
            GroupTable,
            UserGroupTable,
            RefreshTokenTable,
            GroupInviteTable
        )

        SchemaUtils.createMissingTablesAndColumns(
            CategoryTable,
            LocationTable,
            ProductTable,
            StockBatchTable,
            DeletedTable,

            UserTable,
            GroupTable,
            UserGroupTable,
            RefreshTokenTable,
            GroupInviteTable
        )

        try {
            exec("ALTER TABLE product ALTER COLUMN category_id DROP NOT NULL;")
        } catch (_: Exception) {}
    }
}
