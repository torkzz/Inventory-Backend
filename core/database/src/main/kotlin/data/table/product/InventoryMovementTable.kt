package data.table.product

import data.table.user.GroupTable
import org.jetbrains.exposed.v1.core.ReferenceOption
import org.jetbrains.exposed.v1.core.Table

object InventoryMovementTable : Table("inventory_movement") {
    val movementId = varchar("movement_id", 64)
    val productId = reference(
        "product_id",
        ProductTable.productId,
        onDelete = ReferenceOption.CASCADE
    )
    val groupId = reference(
        "group_id",
        GroupTable.groupId,
        onDelete = ReferenceOption.CASCADE
    )
    val movementType = varchar("movement_type", 32) // STOCK_IN, STOCK_OUT, ADJUSTMENT, INITIAL_BALANCE
    val quantity = integer("quantity")
    val unit = varchar("unit", 64).default("pcs")
    val reason = varchar("reason", 255).nullable()
    val movementSource = varchar("source", 32).default("SYSTEM") // ANDROID, WEB, SYSTEM
    val deviceId = varchar("device_id", 64).nullable()
    val createdBy = varchar("created_by", 64).nullable()
    val createdAt = long("created_at")
    val serverUpdatedAt = long("server_updated_at")

    override val primaryKey = PrimaryKey(movementId)

    init {
        index(false, groupId, productId)
        index(false, groupId, serverUpdatedAt)
    }
}
