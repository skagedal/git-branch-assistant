package tech.skagedal.assistant

import java.nio.file.FileSystem
import java.nio.file.Path
import java.util.regex.Matcher

fun FileSystem.home() = getPath(System.getProperty("user.home"))

fun FileSystem.assistantDirectory() = home().resolve(".git-branch-assistant")
fun FileSystem.assistantDataDirectory() = assistantDirectory().resolve("data")
fun FileSystem.logsDirectory() = assistantDirectory().resolve("logs")

fun FileSystem.pathWithShellExpansions(directory: String) = getPath(
    directory.replaceFirst(Regex("^~"), Matcher.quoteReplacement(System.getProperty("user.home")))
)

fun Path.isGloballyIgnored() = fileName.toString().equals(".DS_Store", true)