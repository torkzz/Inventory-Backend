package domain.model

data class ProductParams(
    val productId: ProductId,
    val categoryId: CategoryId?,
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
    val batches: List<StockBatchParams> = emptyList(),
    val createdAt: Long,
    val lastUpdated: Long
)
