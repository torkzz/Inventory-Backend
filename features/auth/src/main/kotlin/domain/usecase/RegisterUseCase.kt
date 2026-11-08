package domain.usecase

import domain.model.AuthTokens
import domain.util.hashWithSHA256
import model.User
import model.UserId
import domain.model.UserSession
import domain.repository.AuthRepository
import domain.security.PasswordHasher
import domain.security.TokenProvider
import model.AppRole
import model.Group
import model.GroupId
import model.GroupWithRole
import model.MembershipStatus
import result.DomainResult
import result.ErrorType
import validation.ValidationRules
import java.util.UUID

class RegisterUseCase (
    private val authRepository: AuthRepository,
    private val passwordHasher: PasswordHasher,
    private val tokenProvider: TokenProvider,
) {
    suspend operator fun invoke(
        name: String,
        email: String,
        rawPassword: String
    ): DomainResult<UserSession> {
        val sanitizedEmail = email.trim().lowercase()
        val sanitizedName = name.trim()

        val errors = mutableListOf<String>()

        if (sanitizedName.length > 128) {
            errors.add("Name cannot exceed 128 characters")
        }

        errors.addAll(
            ValidationRules.validateEmail(
                item = sanitizedEmail,
                getId = { "Email" },
                getValue = { it },
            )
        )
        errors.addAll(
            ValidationRules.validatePassword(
                item = rawPassword,
                getId = { "Password" },
                getValue = { it },
            )
        )

        if (errors.isNotEmpty()) {
            return DomainResult.Error(
                errors.joinToString("\n"),
                ErrorType.BAD_REQUEST
            )
        }

        val hashPassword = passwordHasher.hashPassword(rawPassword)

        val timeStamp = System.currentTimeMillis()
        val newUser = User(
            userId = UserId(UUID.randomUUID().toString()),
            email = sanitizedEmail,
            passHash = hashPassword,
            name = sanitizedName,
            avatarUrl = null,
            phoneNumber = null,
            jobTitle = null,
            locale = null,
            timeZone = null,
            isActive = true,
            createdAt = timeStamp,
            lastUpdated = timeStamp
        )

        val newGroup = Group(
            groupId = GroupId("Group-${UUID.randomUUID()}"),
            name = "${sanitizedName}'s Inventory",
            description = null,
            address = null,
            totalMembers = 1,
            totalProducts = 0,
            createdAt = timeStamp,
            lastUpdated = timeStamp
        )

        return when(
            val result = authRepository.registerUser(newUser, newGroup)
        ) {
            is DomainResult.Success -> {
                val createdUser = result.data
                val accessToken = tokenProvider.generateAccessToken(createdUser.userId.value)
                val refreshToken = tokenProvider.generateRefreshToken()
                val expiresAt = System.currentTimeMillis() + tokenProvider.refreshTokenValidityMs
                
                authRepository.saveRefreshToken(
                    userId = createdUser.userId,
                    token = refreshToken.hashWithSHA256(),
                    expiresAt = expiresAt
                )

                val newGroupWithRole = GroupWithRole(
                    group = newGroup,
                    role = AppRole.OWNER,
                    status = MembershipStatus.ACTIVE,
                    joinedAt = timeStamp
                )

                DomainResult.Success(
                    UserSession(
                        user = createdUser,
                        groups = listOf(newGroupWithRole),
                        tokens = AuthTokens(
                            accessToken = accessToken,
                            refreshToken = refreshToken
                        )
                    )
                )
            }
            is DomainResult.Error -> result
        }
    }
}