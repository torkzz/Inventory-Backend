package api.dto.response

import kotlinx.serialization.Serializable

@Serializable
data class ProductResponse (
    val productId: String,
    val groupId: String,
    val category: CategoryResponse? = null,
    val name: String,
    val brand: String? = null,
    val description: String?,
    val barcode: String?,
    val barcodeFormat: String?,
    val sku: String?,
    val imageUri: String?,
    val unit: String = "pcs",
    val purchasePrice: Double = 0.0,
    val sellingPrice: Double = 0.0,
    val quantity: Int = 0,
    val minimumQuantity: Int = 0,
    val batches: List<StockBatchResponse> = emptyList(),
    val createdAt: Long,
    val lastUpdated: Long
)
