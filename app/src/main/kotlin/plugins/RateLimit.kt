package com.kevinfreyap.plugins

import io.ktor.server.application.Application
import io.ktor.server.application.install
import io.ktor.server.plugins.origin
import io.ktor.server.plugins.ratelimit.RateLimit
import io.ktor.server.plugins.ratelimit.RateLimitName
import kotlin.time.Duration.Companion.seconds

fun Application.configureRateLimit() {
    install(RateLimit) {
        global { 
            rateLimiter(
                limit = 2000,
                refillPeriod = 60.seconds
            )
            requestKey { call -> call.request.origin.remoteHost }
        }

        register (RateLimitName("upload_limit")) {
            rateLimiter(
                limit = 2000,
                refillPeriod = 60.seconds,
            )
            requestKey { call -> call.request.origin.remoteHost }
        }

        register (RateLimitName("auth_limit")) {
            rateLimiter(
                limit = 500,
                refillPeriod = 60.seconds,
            )
            requestKey { call ->
                call.request.headers["X-Forwarded-For"] ?: call.request.local.remoteHost
            }
        }
    }
}
