package api.dto.sync

import kotlinx.serialization.Serializable

@Serializable
data class SyncProductDto(
    val productId: String,
    val categoryId: String? = null,
    val groupId: String,
    val name: String,
    val brand: String? = null,
    val description: String? = null,
    val barcode: String? = null,
    val barcodeFormat: String? = null,
    val sku: String? = null,
    val imageUri: String? = null,
    val unit: String = "pcs",
    val purchasePrice: Double = 0.0,
    val sellingPrice: Double = 0.0,
    val quantity: Int = 0,
    val minimumQuantity: Int = 0,
    val createdAt: Long,
    val lastUpdated: Long
)
