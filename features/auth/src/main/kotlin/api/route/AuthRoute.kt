package api.route

import api.dto.request.LoginRequest
import api.dto.request.LogoutRequest
import api.dto.request.RefreshTokenRequest
import api.dto.request.RegisterRequest
import api.dto.response.AuthResponse
import api.dto.response.TokenResponse
import api.mapper.toResponse
import domain.usecase.AuthUseCases
import io.ktor.http.HttpStatusCode
import io.ktor.resources.Resource
import io.ktor.server.plugins.ratelimit.RateLimitName
import io.ktor.server.plugins.ratelimit.rateLimit
import io.ktor.server.request.receive
import io.ktor.server.resources.post
import io.ktor.server.response.respond
import io.ktor.server.routing.Route
import kotlinx.serialization.Serializable
import org.koin.ktor.ext.inject
import result.DomainResult
import util.toHttpStatusCode

@Serializable
@Resource("/auth")
class AuthResource {
    @Serializable
    @Resource("/register")
    class Register(val parent: AuthResource)

    @Serializable
    @Resource("/login")
    class Login(val parent: AuthResource)

    @Serializable
    @Resource("/local")
    class Local(val parent: AuthResource)

    @Serializable
    @Resource("/refresh")
    class Refresh(val parent: AuthResource)

    @Serializable
    @Resource("/logout")
    class Logout(val parent: AuthResource)
}

fun Route.authRoute() {
    val useCases: AuthUseCases by inject()

    rateLimit(RateLimitName("auth_limit")) {
        post<AuthResource.Register> {
            val request = call.receive<RegisterRequest>()

            when (
                val result = useCases.register(
                    name = request.name,
                    email = request.email,
                    rawPassword = request.password
                )
            ) {
                is DomainResult.Success -> {
                    val session = result.data
                    call.respond(
                        HttpStatusCode.Created,
                        AuthResponse(
                            accessToken = session.tokens.accessToken,
                            refreshToken = session.tokens.refreshToken,
                            user = session.user.toResponse(),
                            groups = session.groups.map { it.toResponse() },
                        )
                    )
                }
                is DomainResult.Error -> call.respond(result.errorType.toHttpStatusCode(), result.message)
            }
        }

        post<AuthResource.Login> {
            val request = call.receive<LoginRequest>()

            when(
                val result = useCases.login(
                    email = request.email,
                    rawPassword = request.password
                )
            ) {
                is DomainResult.Success -> {
                    val session = result.data
                    call.respond(
                        HttpStatusCode.OK,
                        AuthResponse(
                            accessToken = session.tokens.accessToken,
                            refreshToken = session.tokens.refreshToken,
                            user = session.user.toResponse(),
                            groups = session.groups.map { it.toResponse() },
                        )
                    )
                }
                is DomainResult.Error -> call.respond(result.errorType.toHttpStatusCode(), result.message)
            }
        }

        post<AuthResource.Local> {
            when (val result = useCases.localAccess()) {
                is DomainResult.Success -> {
                    val session = result.data
                    call.respond(
                        HttpStatusCode.OK,
                        AuthResponse(
                            accessToken = session.tokens.accessToken,
                            refreshToken = session.tokens.refreshToken,
                            user = session.user.toResponse(),
                            groups = session.groups.map { it.toResponse() },
                        )
                    )
                }
                is DomainResult.Error -> call.respond(result.errorType.toHttpStatusCode(), result.message)
            }
        }

        post<AuthResource.Refresh> {
            val request = call.receive<RefreshTokenRequest>()

            when(
                val result = useCases.refreshToken(request.refreshToken)
            ) {
                is DomainResult.Success -> {
                    call.respond(
                        HttpStatusCode.OK,
                        TokenResponse(
                            accessToken = result.data.accessToken,
                            refreshToken = result.data.refreshToken
                        )
                    )
                }
                is DomainResult.Error -> call.respond(result.errorType.toHttpStatusCode(), result.message)
            }
        }
    }

    post<AuthResource.Logout> {
        val request = call.receive<LogoutRequest>()
        useCases.logout(request.refreshToken)
        call.respond(HttpStatusCode.OK, mapOf("message" to "Successfully logged out"))
    }
}
