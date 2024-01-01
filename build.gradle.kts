plugins {
    application
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.ben.manes.versions)
    alias(libs.plugins.version.catalog.update)
}

repositories {
    mavenCentral()
}

dependencies {
    implementation(libs.logback.classic)
    implementation(libs.kotlinx.coroutines.core)
    implementation(libs.kotlinx.coroutines.jdk8)
    implementation(platform(libs.jackson.bom))
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin")
    implementation("com.fasterxml.jackson.dataformat:jackson-dataformat-yaml")
    implementation("com.fasterxml.jackson.core:jackson-databind")
    implementation(libs.clikt)
    implementation(libs.consoleui)

    compileOnly(libs.slf4j.api)

    testImplementation("org.jetbrains.kotlin:kotlin-test")
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit5")
    testImplementation("org.junit.jupiter", "junit-jupiter", "5.6.0")
}

application {
    mainClass.set("tech.skagedal.assistant.MainKt")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

tasks.named<Test>("test") {
    // Use JUnit Platform for unit tests.
    useJUnitPlatform()
}
