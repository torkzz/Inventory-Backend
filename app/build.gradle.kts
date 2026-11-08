plugins {
    application
    alias(libs.plugins.kotlin.jvm)
    alias(ktorLibs.plugins.ktor)
    alias(libs.plugins.kotlin.serialization)
}

group = "com.kevinfreyap"
version = "1.0.0-SNAPSHOT"

application {
    mainClass = "io.ktor.server.netty.EngineMain"
}

dependencies {
    implementation(project(":core:database"))
    implementation(project(":core:storage"))
    implementation(project(":features:product"))
    implementation(project(":features:auth"))
    implementation(project(":features:user"))

    implementation(ktorLibs.serialization.kotlinx.json)
    implementation(ktorLibs.server.resources)
    implementation(ktorLibs.server.contentNegotiation)
    implementation(ktorLibs.server.core)
    implementation(ktorLibs.server.netty)
    implementation(ktorLibs.server.rateLimit)
    implementation(ktorLibs.server.forwardedHeader)
    implementation(ktorLibs.server.requestValidation)
    implementation(ktorLibs.server.statusPages)
    implementation(ktorLibs.server.websockets)
    implementation(ktorLibs.server.auth.jwt)
    implementation(ktorLibs.server.cors)
    implementation(ktorLibs.server.callLogging)
    implementation(libs.logback.classic)

    implementation(platform(libs.koin.bom))
    implementation(libs.koin.ktor)
    implementation(libs.koin.logger)

    testImplementation(kotlin("test"))
    testImplementation(libs.mockk)
    testImplementation(ktorLibs.server.testHost)
}

kotlin {
    jvmToolchain(21)
}

tasks.test {
    useJUnitPlatform()
}