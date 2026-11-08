package domain.usecase.product

import model.GroupId
import domain.model.Product
import domain.model.ProductParams
import domain.repository.CategoryRepository
import domain.repository.ProductRepository
import domain.usecase.util.StockBatchAssembler
import domain.validation.ProductValidator
import result.DomainResult

class InsertProductUseCase (
    private val productRepository: ProductRepository,
    private val categoryRepository: CategoryRepository,
    private val stockBatchAssembler: StockBatchAssembler,
    private val productValidator: ProductValidator
) {
    suspend operator fun invoke(productParams: ProductParams, groupId: GroupId): DomainResult<Product> {
        val category = if (productParams.categoryId != null) {
            when (val categoryResult = categoryRepository.getCategoryById(productParams.categoryId, groupId)) {
                is DomainResult.Success -> categoryResult.data
                is DomainResult.Error -> return categoryResult
            }
        } else null

        val batches = when (
            val batchesResult = stockBatchAssembler.assembleBatches(productParams.batches, groupId)
        ) {
            is DomainResult.Success -> batchesResult.data
            is DomainResult.Error -> return batchesResult
        }

        val product = Product(
            productId = productParams.productId,
            groupId = groupId,
            category = category,
            name = productParams.name,
            brand = productParams.brand,
            description = productParams.description,
            barcode = productParams.barcode,
            barcodeFormat = productParams.barcodeFormat,
            sku = productParams.sku,
            imageUri = productParams.imageUri,
            unit = productParams.unit,
            purchasePrice = productParams.purchasePrice,
            sellingPrice = productParams.sellingPrice,
            quantity = productParams.quantity,
            minimumQuantity = productParams.minimumQuantity,
            batches = batches,
            createdAt = productParams.createdAt,
            lastUpdated = productParams.lastUpdated
        )

        when(
            val validationResult = productValidator.validateProduct(product)
        ) {
            is DomainResult.Error -> return validationResult
            is DomainResult.Success -> {}
        }

        return productRepository.insertProduct(product, groupId)
    }
}
