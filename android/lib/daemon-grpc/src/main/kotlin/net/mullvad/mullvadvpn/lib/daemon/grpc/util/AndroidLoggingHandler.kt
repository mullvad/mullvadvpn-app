package net.mullvad.mullvadvpn.lib.daemon.grpc.util

import android.util.Log
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
        val maxLength = 30
        val tag = if (name.length > maxLength) name.substring(name.length - maxLength) else name

        try {
            val level: Int = getAndroidLevel(record.getLevel())
            Log.println(level, tag, record.getMessage())
            if (record.thrown != null) {
                Log.println(level, tag, Log.getStackTraceString(record.getThrown()))
            }
        } catch (e: RuntimeException) {
            Log.e("AndroidLoggingHandler", "Error logging message.", e)
        }
    }

    override fun flush() {
        TODO("Not yet implemented")
    }

    override fun close() {
        TODO("Not yet implemented")
    }

    companion object {
        fun reset(rootHandler: Handler) {
            val rootLogger = LogManager.getLogManager().getLogger("")
            val handlers = rootLogger.handlers
            for (handler in handlers) {
                rootLogger.removeHandler(handler)
            }
            rootLogger.addHandler(rootHandler)
        }

        fun getAndroidLevel(level: Level): Int {
            val value = level.intValue()

            return if (value >= Level.SEVERE.intValue()) {
                Log.ERROR
            } else if (value >= Level.WARNING.intValue()) {
                Log.WARN
            } else if (value >= Level.INFO.intValue()) {
                Log.INFO
            } else {
                Log.DEBUG
            }
        }
    }
}
