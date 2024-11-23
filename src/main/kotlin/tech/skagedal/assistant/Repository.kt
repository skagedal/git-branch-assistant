package tech.skagedal.assistant

import tech.skagedal.assistant.configuration.ProcessEnvironment
import java.nio.file.FileSystem
import java.nio.file.Files
import java.nio.file.Path

class Repository(
    val fileSystem: FileSystem
) {
    fun setSuggestedDirectory(suggestedDirectory: Path) {
        val path = pathForRequestedDirectory()
        if (path != null) {
            Files.createDirectories(path.parent)
            Files.writeString(path, suggestedDirectory.toString())
        }
    }

    private fun pathForRequestedDirectory() = ProcessEnvironment.SUGGESTED_CD_FILE?.let {
        fileSystem.pathWithShellExpansions(it)
    }
}