package tech.skagedal.assistant

import org.slf4j.LoggerFactory
import org.springframework.stereotype.Component
import java.nio.file.Path

@Component
class ProcessRunner {
    private val logger = LoggerFactory.getLogger(javaClass)

    fun runBrewUpgrade() {
        runCommand(listOf("brew", "upgrade"))
    }

    fun openUrl(url: String) {
        runCommand(listOf("open", url))
    }

    private fun runCommand(command: List<String>, directory: Path? = null) {
        printCommand(command)
        val returnValue = ProcessBuilder(command)
            .apply { if (directory != null) directory(directory.toFile()) }
            .inheritIO()
            .start()
            .waitFor()
        if (returnValue != 0) {
            throw ProcessExitedUnsuccessfullyException(command, returnValue)
        }
    }

    private fun printCommand(command: List<String>) {
        logger.debug("Running command {}", command.toString())
    }

    fun runEditor(path: Path) {
        runCommand(
            listOf(
                System.getenv("EDITOR"),
                path.toString()
            )
        )
    }

    fun runShellCommand(shellCommand: String, directory: Path?) {
        runCommand(listOf("bash", "-c", shellCommand), directory)
    }
}