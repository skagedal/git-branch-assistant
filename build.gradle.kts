plugins {
    id("org.jetbrains.kotlin.jvm") version "1.9.20"
    application
}

repositories {
    mavenCentral()
}

object Versions {
    val spring = "5.3.4"
    val jackson = "2.11.0"
}

dependencies {
    implementation("org.jetbrains.kotlinx", "kotlinx-coroutines-core", "1.3.8")
    implementation("org.jetbrains.kotlinx", "kotlinx-coroutines-jdk8", "1.3.8")
    implementation("com.fasterxml.jackson.module", "jackson-module-kotlin", Versions.jackson)
    implementation("com.fasterxml.jackson.dataformat", "jackson-dataformat-yaml", Versions.jackson)
    implementation("com.fasterxml.jackson.core", "jackson-databind", Versions.jackson)
    implementation("org.slf4j", "slf4j-api", "1.7.30")
    implementation("ch.qos.logback", "logback-classic", "1.2.3")
    implementation("com.github.ajalt", "clikt", "2.4.0")
    implementation("de.codeshelf.consoleui", "consoleui", "0.0.13")

    testImplementation("org.jetbrains.kotlin", "kotlin-test")
    testImplementation("org.jetbrains.kotlin", "kotlin-test-junit5")
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
