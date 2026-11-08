package api.dto.sync

import api.dto.movement.InventoryMovementDto
import api.dto.response.CategoryResponse
import api.dto.response.LocationResponse
import kotlinx.serialization.Serializable

@Serializable
data class SyncPullResponseDto(
    val categories: List<CategoryResponse>,
    val locations: List<LocationResponse>,
    val products: List<SyncProductDto>,
    val batches: List<SyncStockBatchDto>,
    val movements: List<InventoryMovementDto> = emptyList(),

    val deletedCategories: List<String>,
    val deletedLocations: List<String>,
    val deletedProducts: List<String>,
    val deletedBatches: List<String>,

    val groupId: String,

    val serverTimeStamp: Long,
)
