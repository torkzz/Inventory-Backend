package di

import api.security.JwtConfig
import data.repository.AuthRepositoryImpl
import data.security.Argon2PasswordHasher
import domain.repository.AuthRepository
import domain.security.PasswordHasher
import domain.security.TokenProvider
import domain.usecase.AuthUseCases
import domain.usecase.DeleteExpiredTokensUseCase
import domain.usecase.LocalAccessUseCase
import domain.usecase.LoginUserUseCase
import domain.usecase.LogoutUseCase
import domain.usecase.RefreshTokenUseCase
import domain.usecase.RegisterUseCase
import io.ktor.server.config.ApplicationConfig
import org.koin.dsl.module
import util.getSecret

fun authModule(config: ApplicationConfig) = module {
    //// Api Layer
    single {
        JwtConfig(
            secret = getSecret("jwt_secret", "JWT_SECRET"),
            issuer = config.property("jwt.issuer").getString(),
            audience = config.property("jwt.audience").getString(),
        )
    }

    //// Data Layer
    single<AuthRepository> { AuthRepositoryImpl() }
    single<PasswordHasher> {
        val pepper = getSecret("auth_pepper", "AUTH_PEPPER_SECRET")
        Argon2PasswordHasher(pepper)
    }

    //// Domain Layer
    single <TokenProvider> { get<JwtConfig>() }

    single { RegisterUseCase(get(), get(), get()) }
    single { LoginUserUseCase(get(), get(), get(), get()) }
    single { LocalAccessUseCase(get(), get(), get(), get()) }
    single { RefreshTokenUseCase(get(), get()) }
    single { LogoutUseCase(get()) }
    factory {
        AuthUseCases(
            register = get(),
            login = get(),
            localAccess = get(),
            refreshToken = get(),
            logout = get(),
        )
    }

    factory { DeleteExpiredTokensUseCase(get()) }
}
