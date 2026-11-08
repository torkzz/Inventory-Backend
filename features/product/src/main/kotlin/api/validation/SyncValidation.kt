package api.validation

import api.dto.sync.SyncPayloadDto
import api.dto.sync.SyncProductDto
import api.dto.sync.SyncStockBatchDto

fun SyncPayloadDto.validate(): List<String> {
    val errors = mutableListOf<String>()

    createdCategories.forEach { errors.addAll(it.validate()) }
    updatedCategories.forEach { errors.addAll(it.validate()) }
    deletedCategories.forEach {
        if (it.isBlank()) errors.add("A deleted Category ID is blank")
    }

    createdLocations.forEach { errors.addAll(it.validate()) }
    updatedLocations.forEach { errors.addAll(it.validate()) }
    deletedLocations.forEach {
        if (it.isBlank()) errors.add("A deleted Location ID is blank")
    }

    createdProduct.forEach { errors.addAll(it.validate()) }
    updatedProduct.forEach { errors.addAll(it.validate()) }
    deletedProduct.forEach {
        if (it.isBlank()) errors.add("A deleted Product ID is blank")
    }

    createdBatches.forEach { errors.addAll(it.validate()) }
    updatedBatches.forEach { errors.addAll(it.validate()) }
    deletedBatches.forEach {
        if (it.isBlank()) errors.add("A deleted Batch ID is blank")
    }

    return errors
}

fun SyncProductDto.validate(): List<String> {
    val errors = mutableListOf<String>()

    if (productId.isBlank()) errors.add("ProductId cannot be empty")
    if (categoryId != null && categoryId.isBlank()) errors.add("CategoryId cannot be empty if specified")
    if (name.isBlank()) errors.add("Product name cannot be empty")

    if (minimumQuantity < 0) errors.add("Minimum quantity cannot be lower than 0")
    if (createdAt < 0L) errors.add("Invalid createdAt timestamp")
    if (lastUpdated < 0L) errors.add("Invalid last updated")

    return errors
}

fun SyncStockBatchDto.validate(): List<String> {
    val errors = mutableListOf<String>()

    if (batchId.isBlank()) errors.add("BatchId cannot be empty")
    if (productId.isBlank()) errors.add("ProductId cannot be empty")
    if (locationId.isBlank()) errors.add("LocationId cannot be empty")
    if (quantity < 0) errors.add("Quantity cannot be lower than 0")
    if (price < 0.0) errors.add("Price cannot be lower than 0")
    if (createdAt < 0L) errors.add("Invalid createdAt timestamp")
    if (lastUpdated < 0L) errors.add("Invalid last updated")

    return errors
}