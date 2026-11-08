package domain.usecase

import domain.repository.AuthRepository
import domain.util.hashWithSHA256

class LogoutUseCase (
    private val authRepository: AuthRepository
) {
    suspend operator fun invoke(refreshToken: String) {
        authRepository.revokeRefreshToken(refreshToken.hashWithSHA256())
    }
}
