package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import co.touchlab.kermit.Logger
import co.touchlab.kermit.Severity
import java.util.logging.Handler
import java.util.logging.Level
import java.util.logging.LogManager
import java.util.logging.LogRecord

// Based on:
// https://stackoverflow.com/questions/4561345/how-to-configure-java-util-logging-on-android

/** Make JUL work on Android. */
class AndroidLoggingHandler : Handler() {
    override fun publish(record: LogRecord) {
        if (!super.isLoggable(record)) return

        val name = record.loggerName
        val tag = name.take(MAX_TAG_LENGTH)

        val severity = record.level.toSeverity()
        Logger.log(
            severity = severity,
            tag = tag,
            message = record.message,
            throwable = record.thrown,
        )
    }

    override fun flush() {
        // Do nothing
    }

    override fun close() {
        // No-op, not required since we have nothing to close
    }

    companion object {
        const val MAX_TAG_LENGTH = 30

        fun reset(rootHandler: Handler) {
            val rootLogger = LogManager.getLogManager().getLogger("")
            for (handler in rootLogger.handlers) {
                rootLogger.removeHandler(handler)
            }
            rootLogger.addHandler(rootHandler)
        }
    }
}

private fun Level.toSeverity(): Severity =
    when (this) {
        Level.SEVERE -> Severity.Error
        Level.WARNING -> Severity.Warn
        Level.INFO -> Severity.Info
        else -> Severity.Debug
    }
