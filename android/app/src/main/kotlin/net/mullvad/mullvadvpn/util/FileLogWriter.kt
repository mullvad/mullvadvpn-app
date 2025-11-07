package net.mullvad.mullvadvpn.util

import co.touchlab.kermit.LogWriter
import co.touchlab.kermit.Severity
import java.io.BufferedWriter
import java.io.IOException
import java.nio.channels.FileChannel
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.StandardCopyOption
import java.nio.file.StandardOpenOption
import java.time.OffsetDateTime
import java.time.ZoneOffset
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import kotlin.io.path.createFile
import kotlin.io.path.deleteExisting
import kotlin.io.path.exists
import kotlin.io.path.fileSize
import kotlin.io.path.getLastModifiedTime
import kotlin.io.path.listDirectoryEntries
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * THREAD SAFETY: This class must be thread-safe because any thread can call the Kermit logger
 * (which is itself thread-safe). This is done by only accessing mutable state via a Mutex.
 */
class FileLogWriter(
    private val logDir: Path,
    private val scope: CoroutineScope,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
    private val maxFileCount: Int = MAX_FILE_COUNT,
    private val maxTotalSizeBytes: Long = MAX_TOTAL_SIZE_BYTES,
    private val truncateKeepPercentage: Double = TRUNCATE_KEEP_PERCENTAGE,
    private val checkSizeLimitAfter: Int = SIZE_CHECK_AFTER,
) : LogWriter() {

    private var log: FileAndWriter

    private var currentDateComponents: DateComponents

    private var sizeCheckCounter: Int

    private val mutex = Mutex()

    init {
        synchronized(this) {
            Files.createDirectories(logDir)
            currentDateComponents = DateComponents.utcNow()
            log = FileAndWriter.create(logDir.logFile(currentDateComponents))
            sizeCheckCounter = 0
            checkRemoveOldLog()
            // If the app exited while truncating we may have temporary files we should delete
            logDir.listDirectoryEntries("*.tmp").forEach { it.deleteExisting() }
        }
    }

    override fun log(severity: Severity, message: String, tag: String, throwable: Throwable?) {

        scope.launch(dispatcher) {
            val logTimeStamp = DateTimeFormatter.ISO_LOCAL_DATE_TIME.format(ZonedDateTime.now())

            mutex.withLock {
                try {
                    checkLogRotation()
                    log.writer.appendLine("$logTimeStamp ${severity.name.first()}: $message")
                    throwable?.let { log.writer.appendLine(it.stackTraceToString()) }
                    log.writer.flush()
                } catch (e: IOException) {
                    android.util.Log.e("mullvad", "Error writing to log file", e)
                }
            }
        }
    }

    private fun checkLogRotation() {

        fun rotateLog(now: DateComponents) {
            log.writer.close()
            log = FileAndWriter.create(logDir.logFile(now))
            currentDateComponents = now
            checkRemoveOldLog()
        }

        fun rotateLogSizeLimitIfNeeded() {
            val allLogs = logDir.listDirectoryEntries()
            val totalSizeBytes = allLogs.fold(0L) { acc, path -> acc + path.fileSize() }
            if (totalSizeBytes <= maxTotalSizeBytes) return

            if (allLogs.size == 1) {
                // We only have one log file but it is too big, so we need to truncate it
                val tmpFile = logDir.resolve("${log.logFilePath.fileName}.tmp")
                if (!tmpFile.exists()) tmpFile.createFile()

                log.writer.close()
                val bytesToKeep = (maxTotalSizeBytes * truncateKeepPercentage).toLong()
                copyNBytesFromEnd(log.logFilePath, tmpFile, bytesToKeep)
                Files.move(tmpFile, log.logFilePath, StandardCopyOption.REPLACE_EXISTING)
                log = FileAndWriter.create(log.logFilePath)
            } else {
                val oldest = allLogs.minBy { it.getLastModifiedTime() }
                oldest.deleteExisting()
            }
        }

        val now = DateComponents.utcNow()
        if (now != currentDateComponents) {
            // The day has changed, so we need to rotate the log file
            rotateLog(now)
        }

        sizeCheckCounter += 1
        if (sizeCheckCounter >= checkSizeLimitAfter) {
            sizeCheckCounter = 0
            rotateLogSizeLimitIfNeeded()
        }
    }

    private fun checkRemoveOldLog() {
        val allLogs = logDir.listDirectoryEntries()
        if (allLogs.size > maxFileCount) {
            val oldest = allLogs.minBy { it.getLastModifiedTime() }
            oldest.deleteExisting()
        }
    }

    private fun Path.logFile(components: DateComponents): Path {
        val year = components.year
        val month = components.month.toString().padStart(2, '0')
        val day = components.day.toString().padStart(2, '0')
        return resolve("app_log_$year-$month-$day.log")
    }

    private fun copyNBytesFromEnd(source: Path, dest: Path, n: Long) {
        FileChannel.open(source, StandardOpenOption.READ).use { inChannel ->
            FileChannel.open(
                    dest,
                    StandardOpenOption.CREATE,
                    StandardOpenOption.WRITE,
                    StandardOpenOption.TRUNCATE_EXISTING,
                )
                .use { outChannel ->
                    val start = inChannel.size() - n
                    inChannel.transferTo(start, n, outChannel)
                }
        }
    }

    companion object {
        private const val MAX_FILE_COUNT = 7
        private const val MAX_TOTAL_SIZE_BYTES = 1024L * 1024L * 2L // 2 MB
        private const val TRUNCATE_KEEP_PERCENTAGE = 0.8

        // As an optimization only check if the log size exceeds the limit after this many log
        // writes
        private const val SIZE_CHECK_AFTER = 100
    }
}

private data class FileAndWriter(val logFilePath: Path, val writer: BufferedWriter) {
    companion object {
        fun create(logFile: Path): FileAndWriter {
            if (!logFile.exists()) logFile.createFile()

            return FileAndWriter(
                logFile,
                Files.newBufferedWriter(logFile, StandardOpenOption.APPEND),
            )
        }
    }
}

private data class DateComponents(val year: Int, val month: Int, val day: Int) {
    companion object {
        fun utcNow(): DateComponents {
            val now = OffsetDateTime.now(ZoneOffset.UTC)
            return DateComponents(now.year, now.monthValue, now.dayOfMonth)
        }
    }
}
