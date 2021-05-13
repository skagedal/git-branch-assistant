package tech.skagedal.assistant.commands

import com.github.ajalt.clikt.core.CliktCommand
import com.github.ajalt.clikt.core.subcommands

class SimonsAssistant(
    subcommands: List<CliktCommand>
) : CliktCommand(
    name = "simons-assistant"
) {
    init {
        subcommands(subcommands)
    }

    override fun run() = Unit
}