package tech.skagedal.assistant.git

import java.nio.file.Path

class GitRepo(val dir: Path) {

    fun getNotMergedBranches(): List<String> {
        return git("git", "branch", "--format=%(refname:short)", "--no-merged").linesExceptTrailing()
    }

    fun getBranches(): List<Branch> {
        return git("git", "branch", "--format=%(refname:short):%(upstream:short)")
            .linesExceptTrailing()
            .map { it.split(":") }
            .map { it[0] to it[1] }
            .map { (local, upstream) ->
                Branch(
                    local,
                    if (upstream.isNotBlank())
                        Upstream(upstream, getUpstreamStatus(local, upstream))
                    else
                        null
                )
            }
    }

    fun getUpstreamStatus(local: String, upstream: String): UpstreamStatus {
        val localIsAncestor = isAncestor(local, upstream)
        val upstreamIsAncestor = isAncestor(upstream, local)
        return when {
            localIsAncestor && upstreamIsAncestor -> UpstreamStatus.IDENTICAL
            localIsAncestor && !upstreamIsAncestor -> UpstreamStatus.UPSTREAM_IS_AHEAD_OF_LOCAL
            !localIsAncestor && upstreamIsAncestor -> UpstreamStatus.LOCAL_IS_AHEAD_OF_UPSTREAM
            branchExists(upstream) -> UpstreamStatus.MERGE_NEEDED
            else -> UpstreamStatus.UPSTREAM_IS_GONE
        }
    }

    fun push(refname: String) =
        runInteractivePrintingCommand("git", "push", "origin", refname)
    fun pushCreatingOrigin(refname: String) =
        runInteractivePrintingCommand("git", "push", "--set-upstream", "origin", refname)
    fun rebase(refname: String, upstream: String) =
        runInteractivePrintingCommand("git", "rebase", upstream, refname)
    fun deleteBranchForcefully(branch: String) =
        runInteractivePrintingCommand("git", "branch", "-D", branch)

    fun createPullRequest(refname: String) {
        runInteractivePrintingCommand("gh", "pr", "create", "--head", refname)
    }

    // For some reason --prune-tags causes problems with me in some repos that I haven't yet figured out.
    fun fetchAndPrune() = git("git", "fetch", "--prune" /*, "--prune-tags" */)

    private fun isAncestor(local: String, upstream: String) =
        truthy("git", "merge-base", "--is-ancestor", local, upstream)

    private fun branchExists(branch: String) =
        truthy("git", "rev-parse", "--quiet", "--verify", branch)

    fun getAllRemoteBranches(): List<String> {
        return git("git", "branch", "--remotes", "--format=%(refname:short)").linesExceptTrailing()
    }

    fun showLog(branch: String) {
        runInteractive("tig", branch)
    }

    fun checkoutBranchAndEnterShell(branch: String) {
        checkoutBranch(branch)
        runInteractive("zsh")
    }

    fun checkoutBranch(branch: String) {
        git("git", "checkout", branch)
    }

    fun checkoutFirstAvailableBranch(branches: List<String>) {
        if (branches.isEmpty()) {
            throw BranchNotAvailable()
        }
        try {
            checkoutBranch(branches.first())
        } catch (e: NonZeroGitExitCode) {
            checkoutFirstAvailableBranch(branches.drop(1))
        }
    }

    private fun truthy(vararg command: String) =
        ProcessBuilder(*command)
            .directory(dir.toFile())
            .start()
            .waitFor() == 0

    private fun git(vararg command: String): String {
        val process = ProcessBuilder(*command)
            .directory(dir.toFile())
            .start()
        val exitCode = process.waitFor()
        val output = process.inputStream
            .readAllBytes()
            .toString(Charsets.UTF_8)
        if (exitCode != 0) {
            throw NonZeroGitExitCode(exitCode, output)
        }
        return output
    }

    private fun runInteractivePrintingCommand(vararg command: String) {
        println(command.joinToString(" "))
        runInteractive(*command)
    }

    private fun runInteractive(vararg command: String) {
        val process = ProcessBuilder(*command)
            .directory(dir.toFile())
            .inheritIO()
            .start()
        process.waitFor()
    }

    private fun CharSequence.linesExceptTrailing() = lines().dropLastWhile { it.isEmpty() }
}