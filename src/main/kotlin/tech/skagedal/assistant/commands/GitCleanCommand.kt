package tech.skagedal.assistant.commands

import com.github.ajalt.clikt.core.CliktCommand
import tech.skagedal.assistant.TaskResult
import tech.skagedal.assistant.commands.GitCleanCommand.BranchAction.*
import tech.skagedal.assistant.git.Branch
import tech.skagedal.assistant.git.GitRepo
import tech.skagedal.assistant.git.UpstreamStatus.IDENTICAL
import tech.skagedal.assistant.git.UpstreamStatus.LOCAL_IS_AHEAD_OF_UPSTREAM
import tech.skagedal.assistant.git.UpstreamStatus.MERGE_NEEDED
import tech.skagedal.assistant.git.UpstreamStatus.UPSTREAM_IS_AHEAD_OF_LOCAL
import tech.skagedal.assistant.git.UpstreamStatus.UPSTREAM_IS_GONE
import tech.skagedal.assistant.ui.UserInterface
import java.nio.file.FileSystem
import java.nio.file.Path

class GitCleanCommand(val fileSystem: FileSystem, val userInterface: UserInterface) : CliktCommand(name = "git-clean") {
    override fun run() {
        val repo = GitRepo(fileSystem.getPath("."))
        handle(repo, repo.getBranches())
    }

    enum class BranchAction(val description: String) {
        PUSH("Push to origin"),
        PUSH_CREATING_ORIGIN("Push to create origin"),
        CREATE_PR("Create pull request"),
        REBASE("Rebase onto origin"),
        DELETE("Delete it"),
        LOG("Show git log"),
        SHELL("Exit to shell with branch checked out"),
        NOTHING("Do nothing")
    }

    fun handle(repo: GitRepo, branches: List<Branch>): TaskResult =
        branches.fold(TaskResult.Proceed as TaskResult) { result, branch ->
            when(result) {
                is TaskResult.Proceed -> handle(repo, branch)
                else -> result
            }
        }

    fun handle(repo: GitRepo, branch: Branch): TaskResult {
        return branch.upstream?.let { upstream ->
            when (upstream.status) {
                IDENTICAL ->
                    TaskResult.Proceed
                UPSTREAM_IS_AHEAD_OF_LOCAL -> {
                    repo.rebase(branch.refname, upstream.name)
                    TaskResult.Proceed
                }
                LOCAL_IS_AHEAD_OF_UPSTREAM ->
                    selectAction(
                        repo, branch, "Branch is ahead of upstream", listOf(
                            PUSH, LOG, SHELL, NOTHING
                        )
                    )
                MERGE_NEEDED ->
                    selectAction(
                        repo, branch, "Different commits on local and upstream", listOf(
                            REBASE, LOG, DELETE, SHELL, NOTHING
                        )
                    )
                UPSTREAM_IS_GONE ->
                    selectAction(
                        repo, branch, "Upstream is set, but it is gone", listOf(
                            DELETE, LOG, SHELL, NOTHING
                        )
                    )
            }
        } ?: selectAction(
            repo, branch, "Branch has no upstream", listOf(
                CREATE_PR, PUSH_CREATING_ORIGIN, DELETE, LOG, SHELL, NOTHING
            )
        )
    }

    private fun selectAction(repo: GitRepo, branch: Branch, message: String, actions: List<BranchAction>): TaskResult {
        while (true) {
            val action = userInterface.pickOne<BranchAction>("${repo.dir.fileName}:${branch.refname}: $message") {
                for (action in actions) {
                    choice(action, action.description)
                }
            }
            val actionResult = performAction(repo, branch, action)
            when (actionResult) {
                ActionResult.Handled ->
                    return TaskResult.Proceed
                ActionResult.NotHandled ->
                    continue
                is ActionResult.ExitToShell ->
                    return TaskResult.ShellActionRequired(actionResult.directory)
            }
        }
    }

    sealed class ActionResult {
        object Handled: ActionResult()
        object NotHandled: ActionResult()
        data class ExitToShell(val directory: Path): ActionResult()
    }

    private fun performAction(repo: GitRepo, branch: Branch, action: BranchAction): ActionResult =
        when (action) {
            CREATE_PR -> {
                repo.createPullRequest(branch.refname)
                ActionResult.Handled
            }
            PUSH -> {
                repo.push(branch.refname)
                ActionResult.Handled
            }
            PUSH_CREATING_ORIGIN -> {
                repo.pushCreatingOrigin(branch.refname)
                ActionResult.Handled
            }
            REBASE -> {
                repo.rebase(branch.refname, branch.upstream!!.name)
                ActionResult.Handled
            }
            DELETE -> {
                repo.checkoutFirstAvailableBranch(listOf<String>("release", "master", "main"))
                repo.deleteBranchForcefully(branch.refname)
                ActionResult.Handled
            }
            LOG -> {
                repo.showLog(branch.refname)
                ActionResult.NotHandled
            }
            SHELL -> {
                repo.checkoutBranch(branch.refname)
                ActionResult.ExitToShell(repo.dir)
            }
            NOTHING -> ActionResult.Handled
        }
}
