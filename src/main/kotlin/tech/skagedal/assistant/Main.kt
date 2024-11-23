package tech.skagedal.assistant

import org.slf4j.Logger
import org.slf4j.LoggerFactory
import tech.skagedal.assistant.commands.GitCleanCommand
import tech.skagedal.assistant.commands.GitReposCommand
import tech.skagedal.assistant.commands.SimonsAssistant
import tech.skagedal.assistant.services.GitReposService
import tech.skagedal.assistant.ui.UserInterface
import java.nio.file.FileSystems

private object Main {
    val logger: Logger by lazy { LoggerFactory.getLogger(javaClass) }

    fun main(args: Array<String>) {
        System.setProperty("slf4j.internal.verbosity", "WARN")
        logger.info("Starting git-branch-assistant")

        val fileSystem = FileSystems.getDefault()
        val assistant = SimonsAssistant(
            listOf(
                GitCleanCommand(fileSystem, UserInterface()),
                GitReposCommand(fileSystem, GitReposService(fileSystem), Repository(fileSystem))
            )
        )
        assistant.main(args)
    }
}

fun main(args: Array<String>) = Main.main(args)

