import org.gradle.api.tasks.testing.logging.TestLogEvent

plugins {
    application
    kotlin("jvm") version "1.6.0"
}

repositories {
    jcenter()
}

object Versions {
    val spring = "5.3.4"
    val jackson = "2.11.0"
}

dependencies {
    implementation(platform("org.jetbrains.kotlin:kotlin-bom"))
    implementation(kotlin("stdlib-jdk8"))

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
    mainClassName = "tech.skagedal.assistant.MainKt"

}

tasks {
    test {
        useJUnitPlatform()
        testLogging {
            events = setOf(
                TestLogEvent.STARTED,
                TestLogEvent.PASSED,
                TestLogEvent.FAILED
            )
            // show standard out and standard error of the test
            // JVM(s) on the console
            showStandardStreams = true
        }
    }
}

java {
    sourceCompatibility = org.gradle.api.JavaVersion.VERSION_11
    targetCompatibility = org.gradle.api.JavaVersion.VERSION_11
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions {
        jvmTarget = "11"
    }
}
