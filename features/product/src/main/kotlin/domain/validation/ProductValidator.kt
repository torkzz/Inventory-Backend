package domain.validation

import domain.model.Product
import domain.model.sync.SyncProduct
import result.DomainResult
import validation.ValidationResult
import validation.ValidationRules

class ProductValidator (
    private val stockBatchValidator: StockBatchValidator,
    private val categoryValidator: CategoryValidator
) {
    fun validateProduct(product: Product): DomainResult<Unit> {
        val errors = mutableListOf<String>()

        errors.addAll(
            validateFields(
                item = product,
                getId = { it.productId.value },
                getName = { it.name },
                getDescription = { it.description },
                getBarcode = { it.barcode },
                getSku = { it.sku },
                getImageUri = { it.imageUri },
                getCreatedAt = { it.createdAt },
                getLastUpdated = { it.lastUpdated }
            )
        )

        product.category?.let { cat ->
            when(
                val result = categoryValidator.validateCategory(cat)
            ) {
                is DomainResult.Error -> errors.add(result.message)
                is DomainResult.Success -> {}
            }
        }

        product.batches.forEach { batch ->
            when(
                val result = stockBatchValidator.validateStockBatch(batch)
            ) {
                is DomainResult.Error -> errors.add(result.message)
                is DomainResult.Success -> {}
            }
        }

        return ValidationResult.formatResult(errors)
    }

    fun validateSyncProduct(product: SyncProduct): DomainResult<Unit> {
        val errors = validateFields(
            item = product,
            getId = { it.productId.value },
            getName = { it.name },
            getDescription = { it.description },
            getBarcode = { it.barcode },
            getSku = { it.sku },
            getImageUri = { it.imageUri },
            getCreatedAt = { it.createdAt },
            getLastUpdated = { it.lastUpdated }
        )

        return ValidationResult.formatResult(errors)
    }

    private fun <T> validateFields(
        item: T,
        getId: (T) -> String,
        getName: (T) -> String,
        getDescription: (T) -> String?,
        getBarcode: (T) -> String?,
        getSku: (T) -> String?,
        getImageUri: (T) -> String?,
        getCreatedAt: (T) -> Long,
        getLastUpdated: (T) -> Long,
    ): List<String> {
        val errors = mutableListOf<String>()

        errors.addAll(
            ValidationRules.validateStringLengthAndFormat(
                item = item,
                getId = getId,
                getFieldName = { "Name" },
                getValue = getName,
                maxLength = 255,
                allowNewLines = false,
                allowSpace = true
            )
        )

        errors.addAll(
            ValidationRules.validateStringLengthAndFormat(
                item = item,
                getId = getId,
                getFieldName = { "Description" },
                getValue = getDescription,
                maxLength = 500,
                allowNewLines = true,
                allowSpace = true
            )
        )

        errors.addAll(
            ValidationRules.validateStringLengthAndFormat(
                item = item,
                getId = getId,
                getFieldName = { "Barcode" },
                getValue = getBarcode,
                maxLength = 64,
                allowNewLines = false,
                allowSpace = false
            )
        )

        errors.addAll(
            ValidationRulesProduct.validateBarcodeAsciiNoSpaces(
                item = item,
                getId = getId,
                getFieldName = { "Barcode" },
                getValue = getBarcode,
            )
        )

        errors.addAll(
            ValidationRules.validateStringLengthAndFormat(
                item = item,
                getId = getId,
                getFieldName = { "SKU" },
                getValue = getSku,
                maxLength = 128,
                allowNewLines = false,
                allowSpace = false
            )
        )

        errors.addAll(
            ValidationRules.validateStringLengthAndFormat(
                item = item,
                getId = getId,
                getFieldName = { "Image Uri" },
                getValue = getImageUri,
                maxLength = 256,
                allowNewLines = false,
                allowSpace = false
            )
        )

        errors.addAll(
            ValidationRules.validateTimestamps(
                item = item,
                getId = getId,
                getCreatedAt = getCreatedAt,
                getLastUpdated = getLastUpdated
            )
        )

        return errors
    }
}