package api.dto.movement

import kotlinx.serialization.Serializable

@Serializable
data class InventoryMovementDto(
    val movementId: String,
    val productId: String,
    val groupId: String,
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

@Serializable
data class CreateMovementRequest(
    val movementId: String? = null,
    val productId: String,
    val movementType: String,
    val quantity: Int,
    val unit: String = "pcs",
    val reason: String? = null,
    val source: String = "WEB",
    val deviceId: String? = null,
    val createdBy: String? = null,
    val createdAt: Long? = null
)
