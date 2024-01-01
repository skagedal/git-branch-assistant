package tech.skagedal.assistant

import ch.qos.logback.classic.Level
import ch.qos.logback.classic.Logger
import ch.qos.logback.classic.LoggerContext
import ch.qos.logback.classic.filter.ThresholdFilter
import ch.qos.logback.classic.layout.TTLLLayout
import ch.qos.logback.classic.spi.Configurator
import ch.qos.logback.classic.spi.ILoggingEvent
import ch.qos.logback.core.ConsoleAppender
import ch.qos.logback.core.FileAppender
import ch.qos.logback.core.encoder.LayoutWrappingEncoder
import ch.qos.logback.core.spi.ContextAwareBase
import tech.skagedal.assistant.configuration.ProcessEnvironment
import java.nio.file.FileSystems

class LoggingConfigurator: ContextAwareBase(), Configurator {
    override fun configure(loggerContext: LoggerContext): Configurator.ExecutionStatus {
        val rootLogger = loggerContext.getLogger(Logger.ROOT_LOGGER_NAME)
        if (ProcessEnvironment.DEBUG) {
            rootLogger.addAppender(createConsoleAppender(loggerContext))
        }
        rootLogger.addAppender(createFileAppender(loggerContext))
        return Configurator.ExecutionStatus.NEUTRAL
    }

    private fun createConsoleAppender(loggerContext: LoggerContext) = ConsoleAppender<ILoggingEvent>().apply {
        context = loggerContext
        name = "console"
        encoder = createEncoder(loggerContext)
        addFilter(minimumLevelFilter(Level.INFO))
        start()
    }

    private fun createFileAppender(loggerContext: LoggerContext) = FileAppender<ILoggingEvent>().apply {
        context = loggerContext
        name = "file"
        encoder = createEncoder(loggerContext)
        file = FileSystems.getDefault().logsDirectory().resolve("log.txt").toAbsolutePath().toString()
        start()
    }

    private fun createEncoder(loggerContext: LoggerContext) = LayoutWrappingEncoder<ILoggingEvent>().apply {
        context = loggerContext
        layout = createLayout(loggerContext)
    }

    private fun createLayout(loggerContext: LoggerContext) = TTLLLayout().apply {
        context = loggerContext
        start()
    }

    private fun minimumLevelFilter(level: Level) = ThresholdFilter().apply {
        setLevel(level.levelStr)
        start()
    }
}