plugins {
    kotlin("jvm") version "2.4.0"
    alias(libs.plugins.kotlin.serialization)
}

group = "com.kevinfreyap"
version = "unspecified"

dependencies {
    implementation(project(":core:domain"))
    implementation(project(":core:database"))
    implementation(project(":core:storage"))
    implementation(project(":core:server"))

    implementation(platform(libs.koin.bom))
    implementation(libs.koin.ktor)

    implementation(ktorLibs.server.resources)
    implementation(ktorLibs.server.requestValidation)
    implementation(ktorLibs.server.auth.jwt)
    implementation(libs.kotlinx.serialization)

    implementation(libs.argon2)



    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(21)
}

tasks.test {
    useJUnitPlatform()
}