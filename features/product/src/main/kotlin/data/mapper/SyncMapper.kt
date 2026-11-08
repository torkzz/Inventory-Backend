package data.mapper

import data.table.product.InventoryMovementTable
import data.table.product.ProductTable
import data.table.product.StockBatchTable
import domain.model.BatchId
import domain.model.CategoryId
import domain.model.InventoryMovement
import domain.model.LocationId
import domain.model.MovementId
import domain.model.ProductId
import domain.model.sync.SyncProduct
import domain.model.sync.SyncStockBatch
import model.GroupId
import org.jetbrains.exposed.v1.core.ResultRow

fun ResultRow.toSyncProduct(): SyncProduct =
    SyncProduct(
        productId = ProductId(this[ProductTable.productId]),
        categoryId = this[ProductTable.categoryId]?.let { CategoryId(it) },
        groupId = GroupId(this[ProductTable.groupId]),
        name = this[ProductTable.name],
        brand = this[ProductTable.brand],
        description = this[ProductTable.description],
        barcode = this[ProductTable.barcode],
        barcodeFormat = this[ProductTable.barcodeFormat],
        sku = this[ProductTable.sku],
        imageUri = this[ProductTable.imageUri],
        unit = this[ProductTable.unit],
        purchasePrice = this[ProductTable.purchasePrice],
        sellingPrice = this[ProductTable.sellingPrice],
        quantity = this[ProductTable.quantity],
        minimumQuantity = this[ProductTable.minimumQuantity],
        isArchived = this[ProductTable.isArchived],
        version = this[ProductTable.version],
        createdAt = this[ProductTable.createdAt],
        lastUpdated = this[ProductTable.lastUpdated]
    )

fun ResultRow.toSyncStockBatch(): SyncStockBatch =
    SyncStockBatch(
        batchId = BatchId(this[StockBatchTable.batchId]),
        productId = ProductId(this[StockBatchTable.productId]),
        locationId = LocationId(this[StockBatchTable.locationId]),
        groupId = GroupId(this[StockBatchTable.groupId]),
        quantity = this[StockBatchTable.quantity],
        price = this[StockBatchTable.price],
        expirationDate = this[StockBatchTable.expirationDate],
        supplier = this[StockBatchTable.supplier],
        createdAt = this[StockBatchTable.createdAt],
        lastUpdated = this[StockBatchTable.lastUpdated]
    )

fun ResultRow.toInventoryMovement(): InventoryMovement =
    InventoryMovement(
        movementId = MovementId(this[InventoryMovementTable.movementId]),
        productId = ProductId(this[InventoryMovementTable.productId]),
        groupId = GroupId(this[InventoryMovementTable.groupId]),
        movementType = this[InventoryMovementTable.movementType],
        quantity = this[InventoryMovementTable.quantity],
        unit = this[InventoryMovementTable.unit],
        reason = this[InventoryMovementTable.reason],
        source = this[InventoryMovementTable.movementSource],
        deviceId = this[InventoryMovementTable.deviceId],
        createdBy = this[InventoryMovementTable.createdBy],
        createdAt = this[InventoryMovementTable.createdAt],
        serverUpdatedAt = this[InventoryMovementTable.serverUpdatedAt]
    )
