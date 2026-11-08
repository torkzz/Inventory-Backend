package data.repository

import DatabaseFactory.dbQuery
import data.table.product.CategoryTable
import data.table.product.DeletedTable
import data.table.product.InventoryMovementTable
import data.table.product.LocationTable
import data.table.product.ProductTable
import data.table.product.StockBatchTable
import data.mapper.toCategory
import data.mapper.toInventoryMovement
import data.mapper.toLocation
import data.mapper.toSyncProduct
import data.mapper.toSyncStockBatch
import data.util.EntityType
import domain.model.BatchId
import domain.model.CategoryId
import domain.model.LocationId
import domain.model.ProductId
import domain.model.sync.SyncPayload
import domain.model.sync.SyncPullResponse
import domain.model.sync.SyncPushResponse
import domain.repository.SyncRepository
import model.GroupId
import org.jetbrains.exposed.v1.core.and
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.core.greater
import org.jetbrains.exposed.v1.core.inList
import org.jetbrains.exposed.v1.core.plus
import org.jetbrains.exposed.v1.jdbc.batchUpsert
import org.jetbrains.exposed.v1.jdbc.deleteWhere
import org.jetbrains.exposed.v1.jdbc.selectAll
import org.jetbrains.exposed.v1.jdbc.update
import org.slf4j.LoggerFactory
import result.DomainResult
import result.ErrorType

class SyncRepositoryImpl: SyncRepository {
    private val logger = LoggerFactory.getLogger(SyncRepositoryImpl::class.java)

    override suspend fun pushSync(syncPayload: SyncPayload, groupId: GroupId): DomainResult<SyncPushResponse> {
        return try {
            val response = dbQuery {
                val timeStamp = System.currentTimeMillis()

                val deleted = mutableListOf<Pair<String, String>>()
                syncPayload.deletedBatches.forEach { deleted.add(it.value to EntityType.BATCH.name) }
                syncPayload.deletedProduct.forEach { deleted.add(it.value to EntityType.PRODUCT.name) }
                syncPayload.deletedLocations.forEach { deleted.add(it.value to EntityType.LOCATION.name) }
                syncPayload.deletedCategories.forEach { deleted.add(it.value to EntityType.CATEGORY.name) }

                if (deleted.isNotEmpty()) {
                    DeletedTable.batchUpsert(deleted) { (id, type) ->
                        this[DeletedTable.entityId] = id
                        this[DeletedTable.groupId] = groupId.value
                        this[DeletedTable.entityType] = type
                        this[DeletedTable.deletedAt] = timeStamp
                    }
                }

                // Delete Bottom - Up
                if (syncPayload.deletedBatches.isNotEmpty()) {
                    StockBatchTable.deleteWhere {
                        (StockBatchTable.batchId inList syncPayload.deletedBatches.map { it.value }) and (StockBatchTable.groupId eq groupId.value)
                    }
                }

                if (syncPayload.deletedProduct.isNotEmpty()){
                    ProductTable.deleteWhere {
                        (ProductTable.productId inList syncPayload.deletedProduct.map { it.value }) and (ProductTable.groupId eq groupId.value)
                    }
                }

                if (syncPayload.deletedLocations.isNotEmpty()) {
                    LocationTable.deleteWhere {
                        (LocationTable.locationId inList syncPayload.deletedLocations.map { it.value }) and (LocationTable.groupId eq groupId.value)
                    }
                }

                if (syncPayload.deletedCategories.isNotEmpty()) {
                    CategoryTable.deleteWhere {
                        (CategoryTable.categoryId inList syncPayload.deletedCategories.map { it.value }) and (CategoryTable.groupId eq groupId.value)
                    }
                }

                // Conflict Resolution
                val allCategories = (syncPayload.createdCategories + syncPayload.updatedCategories)
                    .sortedByDescending { it.lastUpdated }
                    .distinctBy { it.categoryId.value }
                val allLocations = (syncPayload.createdLocations + syncPayload.updatedLocations)
                    .sortedByDescending { it.lastUpdated }
                    .distinctBy { it.locationId.value }
                val allProducts = (syncPayload.createdProduct + syncPayload.updatedProduct)
                    .sortedByDescending { it.lastUpdated }
                    .distinctBy { it.productId.value }
                val allBatches = (syncPayload.createdBatches + syncPayload.updatedBatches)
                    .sortedByDescending { it.lastUpdated }
                    .distinctBy { it.batchId.value }

                val incomingIds = allCategories.map { it.categoryId.value } +
                                  allLocations.map { it.locationId.value } +
                                  allProducts.map { it.productId.value } +
                                  allBatches.map { it.batchId.value }

                val deletedMap = mutableMapOf<String, Long>()
                if (incomingIds.isNotEmpty()) {
                    DeletedTable
                        .selectAll()
                        .where { (DeletedTable.entityId inList incomingIds) and (DeletedTable.groupId eq groupId.value) }
                        .forEach { row ->
                            deletedMap[row[DeletedTable.entityId]] = row[DeletedTable.deletedAt]
                        }
                }

                // Create & Update Top - Down
                val categories = allCategories.filter { category ->
                    category.lastUpdated > (deletedMap[category.categoryId.value] ?: 0L)
                }
                if (categories.isNotEmpty()) {
                    CategoryTable.batchUpsert(
                        data = categories,
                        keys = arrayOf(CategoryTable.categoryId)
                    ) { category ->
                        this[CategoryTable.categoryId] = category.categoryId.value
                        this[CategoryTable.groupId] = groupId.value
                        this[CategoryTable.name] = category.name
                        this[CategoryTable.description] = category.description
                        this[CategoryTable.createdAt] = category.createdAt
                        this[CategoryTable.lastUpdated] = category.lastUpdated
                        this[CategoryTable.serverUpdatedAt] = timeStamp
                    }
                }

                val locations = allLocations.filter { location ->
                    location.lastUpdated > (deletedMap[location.locationId.value] ?: 0L)
                }
                if (locations.isNotEmpty()) {
                    LocationTable.batchUpsert(
                        data = locations,
                        keys = arrayOf(LocationTable.locationId)
                    ) { location ->
                        this[LocationTable.locationId] = location.locationId.value
                        this[LocationTable.groupId] = groupId.value
                        this[LocationTable.name] = location.name
                        this[LocationTable.description] = location.description
                        this[LocationTable.locationBarcode] = location.locationBarcode
                        this[LocationTable.createdAt] = location.createdAt
                        this[LocationTable.lastUpdated] = location.lastUpdated
                        this[LocationTable.serverUpdatedAt] = timeStamp
                    }
                }

                val products = allProducts.filter { product ->
                    product.lastUpdated > (deletedMap[product.productId.value] ?: 0L)
                }
                if (products.isNotEmpty()) {
                    ProductTable.batchUpsert(
                        data = products,
                        keys = arrayOf(ProductTable.productId),
                    ) { product ->
                        this[ProductTable.productId] = product.productId.value
                        this[ProductTable.categoryId] = product.categoryId?.value
                        this[ProductTable.groupId] = groupId.value
                        this[ProductTable.name] = product.name
                        this[ProductTable.brand] = product.brand
                        this[ProductTable.description] = product.description
                        this[ProductTable.barcode] = if (product.barcode.isNullOrBlank()) null else product.barcode.trim()
                        this[ProductTable.barcodeFormat] = product.barcodeFormat
                        this[ProductTable.sku] = if (product.sku.isNullOrBlank()) null else product.sku.trim()
                        this[ProductTable.imageUri] = product.imageUri
                        this[ProductTable.unit] = product.unit
                        this[ProductTable.purchasePrice] = product.purchasePrice
                        this[ProductTable.sellingPrice] = product.sellingPrice
                        this[ProductTable.quantity] = product.quantity
                        this[ProductTable.minimumQuantity] = product.minimumQuantity
                        this[ProductTable.isArchived] = product.isArchived
                        this[ProductTable.version] = product.version
                        this[ProductTable.createdAt] = product.createdAt
                        this[ProductTable.lastUpdated] = product.lastUpdated
                        this[ProductTable.serverUpdatedAt] = timeStamp
                    }
                }

                val batches = allBatches.filter { batch ->
                    batch.lastUpdated > (deletedMap[batch.batchId.value] ?: 0L)
                }
                if(batches.isNotEmpty()) {
                    StockBatchTable.batchUpsert(
                        data = batches,
                        keys = arrayOf(StockBatchTable.batchId),
                    ) { batch ->
                        this[StockBatchTable.batchId] = batch.batchId.value
                        this[StockBatchTable.productId] = batch.productId.value
                        this[StockBatchTable.locationId] = batch.locationId.value
                        this[StockBatchTable.groupId] = groupId.value
                        this[StockBatchTable.quantity] = batch.quantity
                        this[StockBatchTable.price] = batch.price
                        this[StockBatchTable.expirationDate] = batch.expirationDate
                        this[StockBatchTable.supplier] = batch.supplier
                        this[StockBatchTable.createdAt] = batch.createdAt
                        this[StockBatchTable.lastUpdated] = batch.lastUpdated
                        this[StockBatchTable.serverUpdatedAt] = timeStamp
                    }
                }

                // Inventory movements ledger (idempotent insert)
                if (syncPayload.movements.isNotEmpty()) {
                    val existingMovementIds = InventoryMovementTable
                        .selectAll()
                        .where {
                            (InventoryMovementTable.movementId inList syncPayload.movements.map { it.movementId.value }) and
                            (InventoryMovementTable.groupId eq groupId.value)
                        }
                        .map { it[InventoryMovementTable.movementId] }
                        .toSet()

                    val newMovements = syncPayload.movements.filter { it.movementId.value !in existingMovementIds }

                    if (newMovements.isNotEmpty()) {
                        InventoryMovementTable.batchUpsert(
                            data = newMovements,
                            keys = arrayOf(InventoryMovementTable.movementId)
                        ) { m ->
                            this[InventoryMovementTable.movementId] = m.movementId.value
                            this[InventoryMovementTable.productId] = m.productId.value
                            this[InventoryMovementTable.groupId] = groupId.value
                            this[InventoryMovementTable.movementType] = m.movementType
                            this[InventoryMovementTable.quantity] = m.quantity
                            this[InventoryMovementTable.unit] = m.unit
                            this[InventoryMovementTable.reason] = m.reason
                            this[InventoryMovementTable.movementSource] = m.source
                            this[InventoryMovementTable.deviceId] = m.deviceId
                            this[InventoryMovementTable.createdBy] = m.createdBy
                            this[InventoryMovementTable.createdAt] = m.createdAt
                            this[InventoryMovementTable.serverUpdatedAt] = timeStamp
                        }

                        // Apply new movements to update product stock balance transactionally
                        for (m in newMovements) {
                            val delta = when (m.movementType.uppercase()) {
                                "STOCK_IN" -> m.quantity
                                "STOCK_OUT" -> -m.quantity
                                "ADJUSTMENT" -> m.quantity
                                "INITIAL_BALANCE" -> m.quantity
                                else -> 0
                            }
                            if (delta != 0) {
                                ProductTable.update({
                                    (ProductTable.productId eq m.productId.value) and
                                    (ProductTable.groupId eq groupId.value)
                                }) {
                                    it[quantity] = quantity + delta
                                    it[lastUpdated] = timeStamp
                                    it[serverUpdatedAt] = timeStamp
                                }
                            }
                        }
                    }
                }

                SyncPushResponse(
                    groupId = groupId,
                    success = true,
                    message = "Sync Success",
                    serverTimeStamp = timeStamp
                )
            }

            DomainResult.Success(response)
        } catch (e: Exception) {
            logger.error("Error pushing sync data", e)
            DomainResult.Error(e.message ?: "Failed to push sync data", ErrorType.UNKNOWN)
        }
    }

    override suspend fun pullSync(updatedAfter: Long, groupId: GroupId): DomainResult<SyncPullResponse> {
        return try {
            val response = dbQuery {
                val timeStamp = System.currentTimeMillis()

                val updatedCategories = CategoryTable
                    .selectAll()
                    .where {
                        (CategoryTable.serverUpdatedAt greater updatedAfter) and (CategoryTable.groupId eq groupId.value)
                    }
                    .map { it.toCategory() }

                val updatedLocations = LocationTable
                    .selectAll()
                    .where {
                        (LocationTable.serverUpdatedAt greater updatedAfter) and (LocationTable.groupId eq groupId.value)
                    }
                    .map { it.toLocation() }

                val updatedProduct = ProductTable
                    .selectAll()
                    .where {
                        (ProductTable.serverUpdatedAt greater updatedAfter) and (ProductTable.groupId eq groupId.value)
                    }
                    .map { it.toSyncProduct() }

                val updatedBatch = StockBatchTable
                    .selectAll()
                    .where {
                        (StockBatchTable.serverUpdatedAt greater updatedAfter) and (StockBatchTable.groupId eq groupId.value)
                    }
                    .map { it.toSyncStockBatch() }

                val updatedMovements = InventoryMovementTable
                    .selectAll()
                    .where {
                        (InventoryMovementTable.serverUpdatedAt greater updatedAfter) and (InventoryMovementTable.groupId eq groupId.value)
                    }
                    .map { it.toInventoryMovement() }

                val deleted = DeletedTable
                    .selectAll()
                    .where {
                        (DeletedTable.deletedAt greater updatedAfter) and (DeletedTable.groupId eq groupId.value)
                    }
                    .toList()

                val deletedCategoryIds = deleted.filter { it[DeletedTable.entityType] == EntityType.CATEGORY.name }
                    .map { CategoryId(it[DeletedTable.entityId]) }
                val deletedLocationIds = deleted.filter { it[DeletedTable.entityType] == EntityType.LOCATION.name }
                    .map { LocationId(it[DeletedTable.entityId]) }
                val deletedProductIds = deleted.filter { it[DeletedTable.entityType] == EntityType.PRODUCT.name }
                    .map { ProductId(it[DeletedTable.entityId]) }
                val deletedBatchIds = deleted.filter { it[DeletedTable.entityType] == EntityType.BATCH.name }
                    .map { BatchId(it[DeletedTable.entityId]) }

                SyncPullResponse(
                    groupId = groupId,
                    categories = updatedCategories,
                    locations = updatedLocations,
                    products = updatedProduct,
                    batches = updatedBatch,
                    movements = updatedMovements,
                    deletedCategories = deletedCategoryIds,
                    deletedLocations = deletedLocationIds,
                    deletedProducts = deletedProductIds,
                    deletedBatches = deletedBatchIds,
                    serverTimeStamp = timeStamp
                )
            }

            DomainResult.Success(response)
        } catch (e: Exception) {
            logger.error("Error pulling sync data", e)
            DomainResult.Error(e.message ?: "Failed to pull sync data", ErrorType.UNKNOWN)
        }
    }
}
