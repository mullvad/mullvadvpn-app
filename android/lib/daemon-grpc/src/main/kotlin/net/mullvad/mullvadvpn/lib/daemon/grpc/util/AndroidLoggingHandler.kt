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
        val maxLength = MAX_LENGTH
        val tag = if (name.length > maxLength) name.substring(name.length - maxLength) else name

        val level = getAndroidLevel(record.level)
        Logger.log(severity = level, tag = tag, message = record.message, throwable = record.thrown)
    }

    override fun flush() {
        // Do nothing
    }

    override fun close() {
        // No-op, not required since we have nothing to close
    }

    companion object {
        const val MAX_LENGTH = 30

        fun reset(rootHandler: Handler) {
            val rootLogger = LogManager.getLogManager().getLogger("")
            val handlers = rootLogger.handlers
            for (handler in handlers) {
                rootLogger.removeHandler(handler)
            }
            rootLogger.addHandler(rootHandler)
        }

        fun getAndroidLevel(level: Level): Severity {
            val value = level.intValue()

            return if (value >= Level.SEVERE.intValue()) {
                Severity.Error
            } else if (value >= Level.WARNING.intValue()) {
                Severity.Warn
            } else if (value >= Level.INFO.intValue()) {
                Severity.Info
            } else {
                Severity.Debug
            }
        }
    }
}
