package data.repository

import DatabaseFactory.dbQuery
import data.table.product.CategoryTable
import data.table.product.DeletedTable
import data.table.product.LocationTable
import data.table.product.ProductTable
import data.table.product.StockBatchTable
import data.mapper.toProduct
import data.mapper.toStockBatch
import data.util.EntityType
import domain.model.Product
import domain.model.ProductId
import domain.repository.ProductRepository
import model.GroupId
import org.jetbrains.exposed.v1.core.and
import result.DomainResult
import result.ErrorType
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.core.inList
import org.jetbrains.exposed.v1.exceptions.ExposedSQLException
import org.jetbrains.exposed.v1.jdbc.batchInsert
import org.jetbrains.exposed.v1.jdbc.deleteWhere
import org.jetbrains.exposed.v1.jdbc.insert
import org.jetbrains.exposed.v1.jdbc.selectAll
import org.jetbrains.exposed.v1.jdbc.update
import org.slf4j.LoggerFactory

class ProductRepositoryImpl: ProductRepository {
    private val logger = LoggerFactory.getLogger(ProductRepositoryImpl::class.java)

    override suspend fun getProducts(groupId: GroupId): DomainResult<List<Product>> = dbQuery {
        try {
            val productRows = ProductTable.leftJoin(CategoryTable)
                .selectAll()
                .where { ProductTable.groupId eq groupId.value }
                .toList()

            if (productRows.isEmpty()) return@dbQuery DomainResult.Success(emptyList())

            val productIds = productRows.map { it[ProductTable.productId] }

            val batchRows = StockBatchTable.innerJoin(LocationTable)
                .selectAll()
                .where { (StockBatchTable.productId inList productIds) and (StockBatchTable.groupId eq groupId.value) }
                .toList()

            val batchByProductId = batchRows.groupBy { it[StockBatchTable.productId] }

            val listProduct = productRows.map { row ->
                val pId = row[ProductTable.productId]

                val productBatches = batchByProductId[pId]?.map { it.toStockBatch() } ?: emptyList()
                row.toProduct(productBatches)
            }

            DomainResult.Success(listProduct)
        } catch (e: Exception) {
            logger.error("Error while fetching products", e)
            DomainResult.Error(e.message ?: "Unknown error", ErrorType.UNKNOWN)
        }
    }

    override suspend fun getProductById(productId: ProductId, groupId: GroupId): DomainResult<Product> = dbQuery {
        try {
            val idValue = productId.value

            val productRow = ProductTable.leftJoin(CategoryTable)
                .selectAll()
                .where { (ProductTable.productId eq idValue) and (ProductTable.groupId eq groupId.value) }
                .singleOrNull()

            if (productRow == null) return@dbQuery DomainResult.Error("Product not found", ErrorType.NOT_FOUND)

            val batchRows = StockBatchTable.innerJoin(LocationTable)
                .selectAll()
                .where { (StockBatchTable.productId eq idValue) and (StockBatchTable.groupId eq groupId.value) }
                .toList()

            val productBatches = batchRows.map { it.toStockBatch() }

            val product = productRow.toProduct(productBatches)

            DomainResult.Success(product)
        } catch (e: Exception) {
            logger.error("Error while fetching product", e)
            DomainResult.Error(e.message ?: "Unknown error", ErrorType.UNKNOWN)
        }
    }

    override suspend fun insertProduct(product: Product, groupId: GroupId): DomainResult<Product> = dbQuery {
        try {
            val timeStamp = System.currentTimeMillis()

            val insertStatement = ProductTable.insert {
                it[productId] = product.productId.value
                it[categoryId] = product.category?.categoryId?.value
                it[this.groupId] = groupId.value
                it[name] = product.name
                it[brand] = product.brand
                it[description] = product.description
                it[barcode] = if (product.barcode.isNullOrBlank()) null else product.barcode.trim()
                it[barcodeFormat] = product.barcodeFormat
                it[sku] = if (product.sku.isNullOrBlank()) null else product.sku.trim()
                it[imageUri] = product.imageUri
                it[unit] = product.unit
                it[purchasePrice] = product.purchasePrice
                it[sellingPrice] = product.sellingPrice
                it[quantity] = product.quantity
                it[minimumQuantity] = product.minimumQuantity
                it[createdAt] = product.createdAt
                it[lastUpdated] = product.lastUpdated
                it[serverUpdatedAt] = timeStamp
            }

            val isProductInserted = insertStatement.insertedCount > 0

            if (isProductInserted && product.batches.isNotEmpty()) {
                StockBatchTable.batchInsert(product.batches) { batch ->
                    this[StockBatchTable.batchId] = batch.batchId.value
                    this[StockBatchTable.productId] = batch.productId.value
                    this[StockBatchTable.locationId] = batch.location.locationId.value
                    this[StockBatchTable.groupId] = groupId.value
                    this[StockBatchTable.quantity] = batch.quantity
                    this[StockBatchTable.price] = batch.price
                    this[StockBatchTable.expirationDate] = batch.expirationDate
                    this[StockBatchTable.supplier] = batch.supplier
                    this[StockBatchTable.lastUpdated] = batch.lastUpdated
                    this[StockBatchTable.serverUpdatedAt] = timeStamp
                }
            }

            DomainResult.Success(product)
        } catch (e: ExposedSQLException) {
            logger.error("Error while inserting product", e)
            val errorMessage = e.message ?: ""

            when{
                errorMessage.contains("product_sku_unique") ->
                    DomainResult.Error("SKU already exists", ErrorType.CONFLICT)
                errorMessage.contains("product_barcode_unique") ->
                    DomainResult.Error("Barcode already exists", ErrorType.CONFLICT)
                errorMessage.contains("product_category_id_fkey") ->
                    DomainResult.Error("Invalid Category ID", ErrorType.NOT_FOUND)
                else ->
                    DomainResult.Error("Failed to save product", ErrorType.UNKNOWN)
            }
        }
    }

    override suspend fun updateProduct(product: Product, groupId: GroupId): DomainResult<Product> = dbQuery {
        try {
            val updatedRows = ProductTable.update({
                (ProductTable.productId eq product.productId.value) and (ProductTable.groupId eq groupId.value)
            }) {
                it[categoryId] = product.category?.categoryId?.value
                it[name] = product.name
                it[brand] = product.brand
                it[description] = product.description
                it[barcode] = if (product.barcode.isNullOrBlank()) null else product.barcode.trim()
                it[barcodeFormat] = product.barcodeFormat
                it[sku] = if (product.sku.isNullOrBlank()) null else product.sku.trim()
                it[imageUri] = product.imageUri
                it[unit] = product.unit
                it[purchasePrice] = product.purchasePrice
                it[sellingPrice] = product.sellingPrice
                it[quantity] = product.quantity
                it[minimumQuantity] = product.minimumQuantity
                it[lastUpdated] = product.lastUpdated
                it[serverUpdatedAt] = System.currentTimeMillis()
            }

            if (updatedRows == 0) {
                return@dbQuery DomainResult.Error("Product Not Found", ErrorType.NOT_FOUND)
            }

            DomainResult.Success(product)
        } catch (e: ExposedSQLException) {
            logger.error("Error while updating product", e)
            val errorMessage = e.message ?: ""

            when{
                errorMessage.contains("product_sku_unique") ->
                    DomainResult.Error("SKU already exists", ErrorType.CONFLICT)
                errorMessage.contains("product_barcode_unique") ->
                    DomainResult.Error("Barcode already exists", ErrorType.CONFLICT)
                errorMessage.contains("product_category_id_fkey") ->
                    DomainResult.Error("Invalid Category ID", ErrorType.NOT_FOUND)
                else ->
                    DomainResult.Error("Failed to save product", ErrorType.UNKNOWN)
            }
        }
    }

    override suspend fun updateProductImage(
        productId: ProductId,
        imageUriPath: String,
        updatedAt: Long,
        groupId: GroupId
    ): DomainResult<Unit> = dbQuery {
        try {
            val updatedRows = ProductTable.update ( {
                (ProductTable.productId eq productId.value) and (ProductTable.groupId eq groupId.value)
            }) {
                it[imageUri] = imageUriPath
                it[lastUpdated] = updatedAt
                it[serverUpdatedAt] = System.currentTimeMillis()
            }

            if (updatedRows == 0) return@dbQuery DomainResult.Error("Product not found", ErrorType.NOT_FOUND)

            DomainResult.Success(Unit)
        } catch (e: ExposedSQLException) {
            logger.error("Error while updating product image", e)
            DomainResult.Error(e.message ?: "Unknown error", ErrorType.UNKNOWN)
        }
    }

    override suspend fun deleteProduct(productId: ProductId, groupId: GroupId): DomainResult<Unit> = dbQuery {
        try {
            val deletedRows = ProductTable.deleteWhere {
                (ProductTable.productId eq productId.value) and (ProductTable.groupId eq groupId.value)
            }

            if (deletedRows == 0) {
                return@dbQuery DomainResult.Error("Product Not Found", ErrorType.NOT_FOUND)
            }

            DeletedTable.insert {
                it[entityId] = productId.value
                it[this.groupId] = groupId.value
                it[entityType] = EntityType.PRODUCT.name
                it[deletedAt] = System.currentTimeMillis()
            }

            DomainResult.Success(Unit)
        } catch (e: Exception) {
            logger.error("Error while deleting product", e)
            DomainResult.Error(e.message ?: "Unknown error", ErrorType.UNKNOWN)
        }
    }

    override suspend fun deleteProductImage(
        productId: ProductId,
        updatedAt: Long,
        groupId: GroupId
    ): DomainResult<Unit> = dbQuery {
        try {
            val updatedRows = ProductTable.update({
                (ProductTable.productId eq productId.value) and (ProductTable.groupId eq groupId.value)
            }) {
                it[imageUri] = null
                it[lastUpdated] = updatedAt
                it[serverUpdatedAt] = System.currentTimeMillis()
            }

            if (updatedRows == 0) return@dbQuery DomainResult.Error("Product Not Found", ErrorType.NOT_FOUND)

            DomainResult.Success(Unit)
        } catch (e: ExposedSQLException) {
            logger.error("Error while deleting product image", e)
            DomainResult.Error(e.message ?: "Unknown error", ErrorType.UNKNOWN)
        }
    }
}
