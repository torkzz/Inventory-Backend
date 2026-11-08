package data.table.product

import data.table.user.GroupTable
import org.jetbrains.exposed.v1.core.ReferenceOption
import org.jetbrains.exposed.v1.core.Table

object ProductTable: Table("product") {
    val productId = varchar("product_id", 64)

    val categoryId = reference(
        "category_id",
        CategoryTable.categoryId,
        ReferenceOption.SET_NULL
    ).nullable()
    val groupId = reference(
        name = "group_id",
        refColumn = GroupTable.groupId,
        onDelete = ReferenceOption.CASCADE
    )

    val name = varchar("name", 255)
    val brand = varchar("brand", 255).nullable()
    val description = text("description").nullable()
    val barcode = varchar("barcode", 64).nullable()
    val barcodeFormat = varchar("barcode_format", 64).nullable()
    val sku = varchar("sku", 128).nullable()
    val imageUri = text("image_uri").nullable()
    val unit = varchar("unit", 64).default("pcs")
    val purchasePrice = double("purchase_price").default(0.0)
    val sellingPrice = double("selling_price").default(0.0)
    val quantity = integer("quantity").default(0)
    val minimumQuantity = integer("minimum_quantity").default(0)
    val isArchived = bool("is_archived").default(false)
    val version = long("version").default(1L)
    val createdAt = long("created_at")
    val lastUpdated = long("last_updated")
    val serverUpdatedAt = long("server_updated_at")

    override val primaryKey: PrimaryKey = PrimaryKey(productId)

    init {
        uniqueIndex(groupId, barcode)
        uniqueIndex(groupId, sku)
    }
}
