package tech.skagedal.assistant.commands

import com.github.ajalt.clikt.core.CliktCommand
import tech.skagedal.assistant.services.GitFetchService
import java.nio.file.FileSystem

class GitFetchCommand(
    private val fileSystem: FileSystem,
    private val gitFetchService: GitFetchService
) : CliktCommand(name = "git-fetch") {
    override fun run() {
        val path = fileSystem.getPath(".")
        gitFetchService.fetchAllGitRepos(path)
    }
}