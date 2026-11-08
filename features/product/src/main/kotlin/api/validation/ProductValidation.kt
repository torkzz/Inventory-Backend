package api.validation

import api.dto.request.ProductRequest

fun ProductRequest.validate(): List<String> {
    val errors = mutableListOf<String>()

    if (productId.isBlank()) errors.add("ProductId cannot be empty")
    if (categoryId != null && categoryId.isBlank()) errors.add("CategoryId cannot be empty if specified")
    if (name.isBlank()) errors.add("Product name cannot be empty")

    if (minimumQuantity < 0) errors.add("Minimum quantity cannot be lower than 0")
    if (createdAt < 0L) errors.add("Invalid createdAt timestamp")
    if (lastUpdated < 0L) errors.add("Invalid last updated")

    batches.forEach { batch ->
        errors.addAll(batch.validate())
    }

    return errors
}
