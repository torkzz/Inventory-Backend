package domain.usecase

import domain.model.AuthTokens
import domain.model.UserSession
import domain.repository.AuthRepository
import domain.security.PasswordHasher
import domain.security.TokenProvider
import domain.util.hashWithSHA256
import model.AppRole
import model.Group
import model.GroupId
import model.GroupWithRole
import model.MembershipStatus
import model.User
import model.UserId
import repository.SharedGroupProvider
import result.DomainResult
import java.util.UUID

class LocalAccessUseCase(
    private val authRepository: AuthRepository,
    private val groupProvider: SharedGroupProvider,
    private val passwordHasher: PasswordHasher,
    private val tokenProvider: TokenProvider,
) {
    suspend operator fun invoke(): DomainResult<UserSession> {
        val email = "pos@local.inventory"
        val existingUser = authRepository.findUserByEmail(email)

        if (existingUser != null) {
            val userGroups = when (val res = groupProvider.getGroupsForUser(existingUser.userId)) {
                is DomainResult.Success -> res.data
                is DomainResult.Error -> emptyList()
            }
            val accessToken = tokenProvider.generateAccessToken(existingUser.userId.value)
            val refreshToken = tokenProvider.generateRefreshToken()
            val expiresAt = System.currentTimeMillis() + tokenProvider.refreshTokenValidityMs

            authRepository.saveRefreshToken(
                userId = existingUser.userId,
                token = refreshToken.hashWithSHA256(),
                expiresAt = expiresAt
            )

            return DomainResult.Success(
                UserSession(
                    user = existingUser,
                    groups = userGroups,
                    tokens = AuthTokens(accessToken = accessToken, refreshToken = refreshToken)
                )
            )
        }

        val timeStamp = System.currentTimeMillis()
        val userId = UserId("user-local-pos")
        val groupId = GroupId("default-grocery-group")

        val newUser = User(
            userId = userId,
            email = email,
            passHash = passwordHasher.hashPassword("LocalStore123"),
            name = "Grocery Manager",
            avatarUrl = null,
            phoneNumber = null,
            jobTitle = "Store Manager",
            locale = null,
            timeZone = null,
            isActive = true,
            createdAt = timeStamp,
            lastUpdated = timeStamp
        )

        val newGroup = Group(
            groupId = groupId,
            name = "Local Grocery Store",
            description = "Main grocery store inventory",
            address = "Local Store",
            totalMembers = 1,
            totalProducts = 0,
            createdAt = timeStamp,
            lastUpdated = timeStamp
        )

        return when (val regResult = authRepository.registerUser(newUser, newGroup)) {
            is DomainResult.Success -> {
                val createdUser = regResult.data
                val accessToken = tokenProvider.generateAccessToken(createdUser.userId.value)
                val refreshToken = tokenProvider.generateRefreshToken()
                val expiresAt = System.currentTimeMillis() + tokenProvider.refreshTokenValidityMs

                authRepository.saveRefreshToken(
                    userId = createdUser.userId,
                    token = refreshToken.hashWithSHA256(),
                    expiresAt = expiresAt
                )

                val groupWithRole = GroupWithRole(
                    group = newGroup,
                    role = AppRole.OWNER,
                    status = MembershipStatus.ACTIVE,
                    joinedAt = timeStamp
                )

                DomainResult.Success(
                    UserSession(
                        user = createdUser,
                        groups = listOf(groupWithRole),
                        tokens = AuthTokens(accessToken = accessToken, refreshToken = refreshToken)
                    )
                )
            }
            is DomainResult.Error -> regResult
        }
    }
}
