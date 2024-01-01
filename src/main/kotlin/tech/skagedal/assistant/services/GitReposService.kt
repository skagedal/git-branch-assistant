package tech.skagedal.assistant.services

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.runBlocking
import tech.skagedal.assistant.TaskResult
import tech.skagedal.assistant.commands.GitCleanCommand
import tech.skagedal.assistant.general.filesInDirectory
import tech.skagedal.assistant.git.Branch
import tech.skagedal.assistant.git.GitRepo
import tech.skagedal.assistant.git.UpstreamStatus
import tech.skagedal.assistant.isGloballyIgnored
import tech.skagedal.assistant.ui.UserInterface
import java.nio.file.FileSystem
import java.nio.file.FileSystems
import java.nio.file.Files
import java.nio.file.Path

class GitReposService(private val fileSystem: FileSystem) {
    fun handleAllGitRepos(path1: Path) = fetchAllResults(path1)
        .filter { it.result != GitResult.Clean }
        .fold(TaskResult.Proceed as TaskResult) { taskResult, repoResult ->
            when (taskResult) {
                is TaskResult.Proceed -> handleNonCleanRepoResult(repoResult)
                else -> taskResult
            }
        }

    private fun fetchAllResults(path: Path) =
        runBlocking {
            path.filesInDirectory().map {
                async(Dispatchers.IO) {
                    ResultWithPath(it, repoResult(it))
                }
            }.awaitAll()
        }

    private fun repoResult(dir: Path): GitResult {
        if (!Files.isDirectory(dir)) {
            return if (dir.isGloballyIgnored()) GitResult.Clean else GitResult.NotDirectory
        }
        val statusResult = gitStatusResult(dir)
        if (statusResult != GitResult.Clean) {
            return statusResult
        }

        val branchesNeedingAction = GitRepo(dir).getBranches().filter { it.needsAction() }
        if (branchesNeedingAction.isNotEmpty()) {
            return GitResult.BranchesNeedingAction(branchesNeedingAction)
        }
        return GitResult.Clean
    }

    private fun gitStatusResult(dir: Path): GitResult {
        val process = ProcessBuilder("git", "status", "--porcelain", "-unormal")
            .directory(dir.toFile())
            .start()
        val exitCode = process.waitFor()
        if (exitCode != 0) {
            return GitResult.NotGitRepository
        }
        val nonEmptyGitStatusResult = process
            .inputStream
            .readAllBytes()
            .isNotEmpty()
        return if (nonEmptyGitStatusResult) GitResult.Dirty else GitResult.Clean
    }

    private fun handleNonCleanRepoResult(it: ResultWithPath): TaskResult {
        val result = it.result
        return when (result) {
            GitResult.Dirty -> {
                System.err.println("Dirty git repository: ${it.path}")
                TaskResult.ShellActionRequired(it.path)
            }
            GitResult.NotGitRepository -> {
                System.err.println("Not a git repository: ${it.path}")
                TaskResult.ShellActionRequired(it.path)
            }
            GitResult.NotDirectory -> {
                System.err.println("Not a directory: ${it.path}")
                TaskResult.ShellActionRequired(it.path.parent)
            }
            GitResult.Clean -> throw IllegalStateException()
            is GitResult.BranchesNeedingAction -> {
                System.err.println("Has branches needing action: ${it.path}")
                val gitClean = GitCleanCommand(FileSystems.getDefault(), UserInterface())
                val gitRepo = GitRepo(it.path)
                gitClean.handle(gitRepo, result.branches)
            }
        }
    }

    sealed class GitResult {
        object Clean: GitResult()
        object Dirty: GitResult()
        object NotGitRepository: GitResult()
        object NotDirectory: GitResult()
        data class BranchesNeedingAction(val branches: List<Branch>): GitResult()
    }

    data class ResultWithPath(
        val path: Path,
        val result: GitResult
    )
}

private fun Branch.needsAction() = upstream?.let { it.status != UpstreamStatus.IDENTICAL } ?: true