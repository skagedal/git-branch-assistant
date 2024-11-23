package tech.skagedal.assistant.configuration

object ProcessEnvironment {
    val DEBUG = System.getenv("DEBUG") == "true"
    val SUGGESTED_CD_FILE: String? = System.getenv("SUGGESTED_CD_FILE")
}