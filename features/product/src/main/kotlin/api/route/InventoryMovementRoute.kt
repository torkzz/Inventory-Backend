package api.route

import DatabaseFactory.dbQuery
import api.dto.movement.CreateMovementRequest
import api.dto.movement.InventoryMovementDto
import data.mapper.toInventoryMovement
import data.table.product.CategoryTable
import data.table.product.InventoryMovementTable
import data.table.product.ProductTable
import io.ktor.http.HttpStatusCode
import io.ktor.resources.Resource
import io.ktor.server.auth.authenticate
import io.ktor.server.request.receive
import io.ktor.server.resources.get
import io.ktor.server.resources.post
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import kotlinx.serialization.Serializable
import model.AppRole
import model.GroupId
import org.jetbrains.exposed.v1.core.and
import org.jetbrains.exposed.v1.core.eq
import org.jetbrains.exposed.v1.core.lessEq
import org.jetbrains.exposed.v1.core.plus
import org.jetbrains.exposed.v1.jdbc.insert
import org.jetbrains.exposed.v1.jdbc.selectAll
import org.jetbrains.exposed.v1.jdbc.update
import org.koin.ktor.ext.inject
import resource.GroupResource
import usecase.VerifyUserGroupRoleUseCase
import util.verifyGroupAccess
import java.util.UUID

@Serializable
@Resource("movements")
class MovementResource(val parent: GroupResource.Id)

@Serializable
@Resource("summary")
class SummaryResource(val parent: GroupResource.Id)

@Serializable
@Resource("archive")
class ArchiveProductResource(val parent: ProductResource.Id)

@Serializable
data class DashboardSummaryDto(
    val totalProducts: Int,
    val totalCategories: Int,
    val lowStockCount: Int,
    val totalStockValue: Double,
    val recentMovements: List<InventoryMovementDto>
)

fun Route.inventoryMovementRoutes() {
    val verifyUserGroupRoleUseCase: VerifyUserGroupRoleUseCase by inject()

    authenticate("auth-jwt") {
        // List movements
        get<MovementResource> { request ->
            val groupId = GroupId(request.parent.groupId)
            call.verifyGroupAccess(groupId = groupId, verifyUserGroupRole = verifyUserGroupRoleUseCase) ?: return@get

            val movements = dbQuery {
                InventoryMovementTable
                    .selectAll()
                    .where { InventoryMovementTable.groupId eq groupId.value }
                    .orderBy(InventoryMovementTable.createdAt, org.jetbrains.exposed.v1.core.SortOrder.DESC)
                    .limit(100)
                    .map { row ->
                        InventoryMovementDto(
                            movementId = row[InventoryMovementTable.movementId],
                            productId = row[InventoryMovementTable.productId],
                            groupId = row[InventoryMovementTable.groupId],
                            movementType = row[InventoryMovementTable.movementType],
                            quantity = row[InventoryMovementTable.quantity],
                            unit = row[InventoryMovementTable.unit],
                            reason = row[InventoryMovementTable.reason],
                            source = row[InventoryMovementTable.movementSource],
                            deviceId = row[InventoryMovementTable.deviceId],
                            createdBy = row[InventoryMovementTable.createdBy],
                            createdAt = row[InventoryMovementTable.createdAt],
                            serverUpdatedAt = row[InventoryMovementTable.serverUpdatedAt]
                        )
                    }
            }

            call.respond(HttpStatusCode.OK, movements)
        }

        // Create movement (STOCK_IN, STOCK_OUT, ADJUSTMENT)
        post<MovementResource> { request ->
            val groupId = GroupId(request.parent.groupId)
            call.verifyGroupAccess(
                groupId = groupId,
                verifyUserGroupRole = verifyUserGroupRoleUseCase,
                allowedRoles = listOf(AppRole.OWNER, AppRole.ADMIN, AppRole.MEMBER)
            ) ?: return@post

            val body = call.receive<CreateMovementRequest>()
            val finalMovementId = body.movementId ?: "mov-${UUID.randomUUID()}"
            val timeStamp = System.currentTimeMillis()
            val finalCreatedAt = body.createdAt ?: timeStamp
            val activeGroupId = groupId

            val created = dbQuery {
                InventoryMovementTable.insert {
                    it[movementId] = finalMovementId
                    it[productId] = body.productId
                    it[InventoryMovementTable.groupId] = activeGroupId.value
                    it[movementType] = body.movementType
                    it[quantity] = body.quantity
                    it[unit] = body.unit
                    it[reason] = body.reason
                    it[movementSource] = body.source
                    it[deviceId] = body.deviceId
                    it[createdBy] = body.createdBy
                    it[createdAt] = finalCreatedAt
                    it[serverUpdatedAt] = timeStamp
                }

                val delta = when (body.movementType.uppercase()) {
                    "STOCK_IN" -> body.quantity
                    "STOCK_OUT" -> -body.quantity
                    "ADJUSTMENT" -> body.quantity
                    "INITIAL_BALANCE" -> body.quantity
                    else -> 0
                }

                if (delta != 0) {
                    ProductTable.update({
                        (ProductTable.productId eq body.productId) and (ProductTable.groupId eq groupId.value)
                    }) {
                        it[quantity] = quantity + delta
                        it[lastUpdated] = timeStamp
                        it[serverUpdatedAt] = timeStamp
                    }
                }

                InventoryMovementDto(
                    movementId = finalMovementId,
                    productId = body.productId,
                    groupId = activeGroupId.value,
                    movementType = body.movementType,
                    quantity = body.quantity,
                    unit = body.unit,
                    reason = body.reason,
                    source = body.source,
                    deviceId = body.deviceId,
                    createdBy = body.createdBy,
                    createdAt = finalCreatedAt,
                    serverUpdatedAt = timeStamp
                )
            }

            call.respond(HttpStatusCode.Created, created)
        }

        // Archive / Unarchive product
        post<ArchiveProductResource> { request ->
            val groupId = GroupId(request.parent.parent.parent.groupId)
            val productId = request.parent.productId

            call.verifyGroupAccess(
                groupId = groupId,
                verifyUserGroupRole = verifyUserGroupRoleUseCase,
                allowedRoles = listOf(AppRole.OWNER, AppRole.ADMIN)
            ) ?: return@post

            val timeStamp = System.currentTimeMillis()
            val updated = dbQuery {
                val current = ProductTable.selectAll()
                    .where { (ProductTable.productId eq productId) and (ProductTable.groupId eq groupId.value) }
                    .singleOrNull()

                if (current == null) return@dbQuery null

                val newArchived = !current[ProductTable.isArchived]
                ProductTable.update({
                    (ProductTable.productId eq productId) and (ProductTable.groupId eq groupId.value)
                }) {
                    it[isArchived] = newArchived
                    it[version] = version + 1L
                    it[lastUpdated] = timeStamp
                    it[serverUpdatedAt] = timeStamp
                }
                newArchived
            }

            if (updated == null) {
                call.respond(HttpStatusCode.NotFound, "Product not found")
            } else {
                call.respond(HttpStatusCode.OK, mapOf("productId" to productId, "isArchived" to updated))
            }
        }

        // Dashboard Summary for Web & Android
        get<SummaryResource> { request ->
            val groupId = GroupId(request.parent.groupId)
            call.verifyGroupAccess(groupId = groupId, verifyUserGroupRole = verifyUserGroupRoleUseCase) ?: return@get

            val summary = dbQuery {
                val products = ProductTable.selectAll()
                    .where { (ProductTable.groupId eq groupId.value) and (ProductTable.isArchived eq false) }
                    .toList()

                val totalProducts = products.size
                val totalCategories = CategoryTable.selectAll()
                    .where { CategoryTable.groupId eq groupId.value }
                    .count()
                    .toInt()

                val lowStockCount = products.count { it[ProductTable.quantity] <= it[ProductTable.minimumQuantity] }
                val totalStockValue = products.sumOf {
                    val q = it[ProductTable.quantity]
                    val p = it[ProductTable.purchasePrice]
                    if (p > 0.0) q * p else 0.0
                }

                val recentMovements = InventoryMovementTable
                    .selectAll()
                    .where { InventoryMovementTable.groupId eq groupId.value }
                    .orderBy(InventoryMovementTable.createdAt, org.jetbrains.exposed.v1.core.SortOrder.DESC)
                    .limit(10)
                    .map { row ->
                        InventoryMovementDto(
                            movementId = row[InventoryMovementTable.movementId],
                            productId = row[InventoryMovementTable.productId],
                            groupId = row[InventoryMovementTable.groupId],
                            movementType = row[InventoryMovementTable.movementType],
                            quantity = row[InventoryMovementTable.quantity],
                            unit = row[InventoryMovementTable.unit],
                            reason = row[InventoryMovementTable.reason],
                            source = row[InventoryMovementTable.movementSource],
                            deviceId = row[InventoryMovementTable.deviceId],
                            createdBy = row[InventoryMovementTable.createdBy],
                            createdAt = row[InventoryMovementTable.createdAt],
                            serverUpdatedAt = row[InventoryMovementTable.serverUpdatedAt]
                        )
                    }

                DashboardSummaryDto(
                    totalProducts = totalProducts,
                    totalCategories = totalCategories,
                    lowStockCount = lowStockCount,
                    totalStockValue = totalStockValue,
                    recentMovements = recentMovements
                )
            }

            call.respond(HttpStatusCode.OK, summary)
        }
    }
}
