package data.mapper

import data.table.product.CategoryTable
import data.table.product.ProductTable
import domain.model.Product
import domain.model.ProductId
import domain.model.StockBatch
import model.GroupId
import org.jetbrains.exposed.v1.core.ResultRow

fun ResultRow.toProduct(batches: List<StockBatch>): Product {
    val category = if (this.getOrNull(ProductTable.categoryId) != null && this.getOrNull(CategoryTable.categoryId) != null) {
        this.toCategory()
    } else null

    return Product(
        productId = ProductId(this[ProductTable.productId]),
        groupId = GroupId(this[ProductTable.groupId]),
        name = this[ProductTable.name],
        brand = this[ProductTable.brand],
        description = this[ProductTable.description],
        category = category,
        barcode = this[ProductTable.barcode],
        barcodeFormat = this[ProductTable.barcodeFormat],
        sku = this[ProductTable.sku],
        imageUri = this[ProductTable.imageUri],
        unit = this[ProductTable.unit],
        purchasePrice = this[ProductTable.purchasePrice],
        sellingPrice = this[ProductTable.sellingPrice],
        quantity = this[ProductTable.quantity],
        minimumQuantity = this[ProductTable.minimumQuantity],
        batches = batches,
        createdAt = this[ProductTable.createdAt],
        lastUpdated = this[ProductTable.lastUpdated],
    )
}
