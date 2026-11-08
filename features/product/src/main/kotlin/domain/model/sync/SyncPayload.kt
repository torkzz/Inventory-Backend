package domain.model.sync

import domain.model.BatchId
import domain.model.Category
import domain.model.CategoryId
import domain.model.InventoryMovement
import domain.model.Location
import domain.model.LocationId
import domain.model.ProductId

data class SyncPayload (
    val createdCategories: List<Category>,
    val updatedCategories: List<Category>,
    val deletedCategories: List<CategoryId>,

    val createdLocations: List<Location>,
    val updatedLocations: List<Location>,
    val deletedLocations: List<LocationId>,

    val createdProduct: List<SyncProduct>,
    val updatedProduct: List<SyncProduct>,
    val deletedProduct: List<ProductId>,

    val createdBatches: List<SyncStockBatch>,
    val updatedBatches: List<SyncStockBatch>,
    val deletedBatches: List<BatchId>,

    val movements: List<InventoryMovement> = emptyList()
)
