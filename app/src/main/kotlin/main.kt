package com.kevinfreyap

import com.kevinfreyap.plugins.configureCors
import com.kevinfreyap.plugins.configureDI
import com.kevinfreyap.plugins.configureLogging
import com.kevinfreyap.plugins.configureProxySupport
import com.kevinfreyap.plugins.configureRateLimit
import com.kevinfreyap.plugins.configureResources
import com.kevinfreyap.plugins.configureSerialization
import com.kevinfreyap.plugins.configureSockets
import io.ktor.server.application.Application

fun main(args: Array<String>) {
    io.ktor.server.netty.EngineMain.main(args)
}

fun Application.module() {
    configureDI()

    initializeDatabaseSchema()

    configureSerialization()
    configureResources()

    configureProxySupport()
    configureCors()
    configureRateLimit()
    configureValidation()
    configureSockets()
    configureSecurity()
    configureBackgroundJobs()

    configureLogging()

    configureRouting()
}