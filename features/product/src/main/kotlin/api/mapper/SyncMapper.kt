package api.mapper

import api.dto.sync.SyncPayloadDto
import api.dto.sync.SyncProductDto
import api.dto.sync.SyncPullResponseDto
import api.dto.sync.SyncPushResponseDto
import api.dto.sync.SyncStockBatchDto
import domain.model.BatchId
import domain.model.CategoryId
import domain.model.LocationId
import domain.model.ProductId
import domain.model.sync.SyncPayload
import domain.model.sync.SyncProduct
import domain.model.sync.SyncPullResponse
import domain.model.sync.SyncPushResponse
import domain.model.sync.SyncStockBatch
import model.GroupId
import util.cleanInlineSpaces

fun SyncProductDto.toDomain(): SyncProduct =
    SyncProduct(
        productId = ProductId(this.productId),
        categoryId = this.categoryId?.let { CategoryId(it) },
        groupId = GroupId(this.groupId),
        name = this.name.cleanInlineSpaces(),
        brand = this.brand?.cleanInlineSpaces(),
        description = this.description?.cleanInlineSpaces(),
        barcode = this.barcode?.trim()?.ifEmpty { null },
        barcodeFormat = this.barcodeFormat?.trim(),
        sku = this.sku?.trim()?.uppercase()?.ifEmpty { null },
        imageUri = this.imageUri?.trim(),
        unit = this.unit.ifBlank { "pcs" },
        purchasePrice = this.purchasePrice,
        sellingPrice = this.sellingPrice,
        quantity = this.quantity,
        minimumQuantity = this.minimumQuantity,
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )

fun SyncStockBatchDto.toDomain(): SyncStockBatch =
    SyncStockBatch(
        batchId = BatchId(this.batchId),
        productId = ProductId(this.productId),
        locationId = LocationId(this.locationId),
        groupId = GroupId(this.groupId),
        quantity = this.quantity,
        price = this.price,
        expirationDate = this.expirationDate,
        supplier = this.supplier?.trim(),
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )

fun SyncPayloadDto.toDomain(groupId: GroupId): SyncPayload =
    SyncPayload(
        createdCategories = this.createdCategories.map { it.toDomain(groupId) },
        updatedCategories = this.updatedCategories.map { it.toDomain(groupId) },
        deletedCategories = this.deletedCategories.map { CategoryId(it) },

        createdLocations = this.createdLocations.map { it.toDomain(groupId) },
        updatedLocations = this.updatedLocations.map { it.toDomain(groupId) },
        deletedLocations = this.deletedLocations.map { LocationId(it) },

        createdProduct = this.createdProduct.map { it.toDomain() },
        updatedProduct = this.updatedProduct.map { it.toDomain() },
        deletedProduct = this.deletedProduct.map { ProductId(it) },

        createdBatches = this.createdBatches.map { it.toDomain() },
        updatedBatches = this.updatedBatches.map { it.toDomain() },
        deletedBatches = this.deletedBatches.map { BatchId(it) }
    )

fun SyncProduct.toDto(): SyncProductDto =
    SyncProductDto(
        productId = this.productId.value,
        categoryId = this.categoryId?.value,
        groupId = this.groupId.value,
        name = this.name,
        brand = this.brand,
        description = this.description,
        barcode = this.barcode,
        barcodeFormat = this.barcodeFormat,
        sku = this.sku,
        imageUri = this.imageUri,
        unit = this.unit,
        purchasePrice = this.purchasePrice,
        sellingPrice = this.sellingPrice,
        quantity = this.quantity,
        minimumQuantity = this.minimumQuantity,
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )

fun SyncStockBatch.toDto(): SyncStockBatchDto =
    SyncStockBatchDto(
        batchId = this.batchId.value,
        productId = this.productId.value,
        locationId = this.locationId.value,
        groupId = this.groupId.value,
        quantity = this.quantity,
        price = this.price,
        expirationDate = this.expirationDate,
        supplier = this.supplier,
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )

fun SyncPushResponse.toDto(): SyncPushResponseDto =
    SyncPushResponseDto(
        groupId = this.groupId.value,
        success = this.success,
        message = this.message,
        serverTimeStamp = this.serverTimeStamp
    )

fun SyncPullResponse.toDto(): SyncPullResponseDto =
    SyncPullResponseDto(
        groupId = this.groupId.value,

        categories = this.categories.map { it.toResponse() },
        locations = this.locations.map { it.toResponse() },
        products = this.products.map { it.toDto() },
        batches = this.batches.map { it.toDto() },

        deletedCategories = this.deletedCategories.map { it.value },
        deletedLocations = this.deletedLocations.map { it.value },
        deletedProducts = this.deletedProducts.map { it.value },
        deletedBatches = this.deletedBatches.map { it.value },
        serverTimeStamp = this.serverTimeStamp
    )
