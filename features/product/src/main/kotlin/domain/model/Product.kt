package domain.model

import model.GroupId

@JvmInline
value class ProductId(val value: String)

data class Product(
    val productId: ProductId,
    val groupId: GroupId,
    val category: Category?,
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
    val isArchived: Boolean = false,
    val version: Long = 1L,
    val batches: List<StockBatch> = emptyList(),
    val createdAt: Long,
    val lastUpdated: Long
)
