package tech.skagedal.assistant.ui

import org.jline.reader.LineReaderBuilder
import org.jline.terminal.TerminalBuilder

class UserInterface {
    class Choices<T>() {
        val choices: MutableList<Pair<T, String>> = mutableListOf()
        fun choice(identifier: T, text: String) {
            choices.add(Pair(identifier, text))
        }
    }

    fun <T> pickOne(message: String, choiceBuilder: Choices<T>.() -> Unit): T {
        val choices = Choices<T>().apply(choiceBuilder).choices

        println(message)
        choices.forEachIndexed { index, pair ->
            println("${index + 1}. ${pair.second}")
        }

        val terminal = TerminalBuilder.builder().build()
        val reader = LineReaderBuilder.builder()
            .terminal(terminal)
            .build()

        while (true) {
            val line = reader.readLine("Select (1-${choices.size}): ")
            val selection = line.toIntOrNull()
            if (selection != null && selection in 1..choices.size) {
                return choices[selection - 1].first
            }
            println("Invalid selection. Please enter a number between 1 and ${choices.size}.")
        }
    }

    fun reportActionTaken(message: String) {
        println(message)
    }

    fun reportError(message: String) {
        println(message)
    }
}