package api.dto.sync

import api.dto.movement.InventoryMovementDto
import api.dto.request.CategoryRequest
import api.dto.request.LocationRequest
import kotlinx.serialization.Serializable

@Serializable
data class SyncPayloadDto(
    val createdCategories: List<CategoryRequest> = emptyList(),
    val updatedCategories: List<CategoryRequest> = emptyList(),
    val deletedCategories: List<String> = emptyList(),

    val createdLocations: List<LocationRequest> = emptyList(),
    val updatedLocations: List<LocationRequest> = emptyList(),
    val deletedLocations: List<String> = emptyList(),

    val createdProduct: List<SyncProductDto> = emptyList(),
    val updatedProduct: List<SyncProductDto> = emptyList(),
    val deletedProduct: List<String> = emptyList(),

    val createdBatches: List<SyncStockBatchDto> = emptyList(),
    val updatedBatches: List<SyncStockBatchDto> = emptyList(),
    val deletedBatches: List<String> = emptyList(),

    val movements: List<InventoryMovementDto> = emptyList()
)
