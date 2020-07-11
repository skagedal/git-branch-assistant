/*
 * This Kotlin source file was generated by the Gradle 'init' task.
 */
package tech.skagedal.next

import java.nio.file.FileSystems

class App(
    val fileSystemLinter: FileSystemLinter,
    val intervalTaskRunner: IntervalTaskRunner
) {
    fun run() {
        fileSystemLinter.run()
        intervalTaskRunner.run()
    }
}

fun main(args: Array<String>) {
    val processRunner = ProcessRunner()
    val fileSystem = FileSystems.getDefault()
    val taskRecords = TaskRecords(fileSystem)

    val fileSystemLinter = FileSystemLinter(
        fileSystem,
        processRunner
    )
    val intervalTaskRunner = IntervalTaskRunner(
        processRunner,
        taskRecords
    )
    val app = App(fileSystemLinter, intervalTaskRunner)

    app.run()
}
