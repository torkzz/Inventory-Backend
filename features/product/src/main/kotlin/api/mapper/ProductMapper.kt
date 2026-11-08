package api.mapper

import api.dto.request.ProductRequest
import api.dto.response.ProductResponse
import domain.model.CategoryId
import domain.model.Product
import domain.model.ProductId
import domain.model.ProductParams
import util.cleanInlineSpaces

fun ProductRequest.toDomainParams(): ProductParams =
    ProductParams(
        productId = ProductId(this.productId),
        categoryId = this.categoryId?.let { CategoryId(it) },
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
        batches = this.batches.map { it.toDomainParams() },
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )

fun Product.toResponse(): ProductResponse =
    ProductResponse(
        productId = this.productId.value,
        groupId = this.groupId.value,
        category = this.category?.toResponse(),
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
        batches = this.batches.map { it.toResponse() },
        createdAt = this.createdAt,
        lastUpdated = this.lastUpdated
    )
