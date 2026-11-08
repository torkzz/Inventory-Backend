package domain.model.sync

import domain.model.CategoryId
import domain.model.ProductId
import model.GroupId

data class SyncProduct (
    val productId: ProductId,
    val categoryId: CategoryId?,
    val groupId: GroupId,
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
    val createdAt: Long,
    val lastUpdated: Long
)
