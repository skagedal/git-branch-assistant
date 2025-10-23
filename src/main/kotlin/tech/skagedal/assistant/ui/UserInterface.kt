package tech.skagedal.assistant.ui

import de.codeshelf.consoleui.prompt.ConsolePrompt
import de.codeshelf.consoleui.prompt.ListResult
import org.fusesource.jansi.AnsiConsole

class UserInterface {
    class Choices<T>() {
        val choices: MutableList<Pair<T, String>> = mutableListOf()
        fun choice(identifier: T, text: String) {
            choices.add(Pair(identifier, text))
        }
    }

    fun <T> pickOne(message: String, choiceBuilder: Choices<T>.() -> Unit): T {
        val choices = Choices<T>().apply(choiceBuilder).choices

        try {
            AnsiConsole.systemInstall()

            val prompt = ConsolePrompt()
            val promptBuilder = prompt.promptBuilder
            val listPromptBuilder = promptBuilder.createListPrompt()
                .name("choice")
                .message(message)
            choices.forEachIndexed { index, pair ->
                listPromptBuilder
                    .newItem(index.toString()).text(pair.second).add()
            }
            listPromptBuilder.addPrompt()
            val result = prompt.prompt(promptBuilder.build())
            val listResult = result["choice"] as ListResult
            val choice = choices[listResult.selectedId.toInt()]
            return choice.first
        } finally {
            AnsiConsole.systemUninstall()
        }
    }

    fun reportActionTaken(message: String) {
        println(message)
    }

    fun reportError(message: String) {
        println(message)
    }
}