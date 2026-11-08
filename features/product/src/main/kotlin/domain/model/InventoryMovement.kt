package domain.model

import model.GroupId

@JvmInline
value class MovementId(val value: String)

data class InventoryMovement(
    val movementId: MovementId,
    val productId: ProductId,
    val groupId: GroupId,
    val movementType: String,
    val quantity: Int,
    val unit: String = "pcs",
    val reason: String? = null,
    val source: String = "SYSTEM",
    val deviceId: String? = null,
    val createdBy: String? = null,
    val createdAt: Long,
    val serverUpdatedAt: Long = 0L
)
