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
        // The synchronized here is to guarantee field visibility for all threads.
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
            val totalSizeBytes = allLogs.sumOf { path -> path.fileSize() }
            if (totalSizeBytes <= maxTotalSizeBytes) return

            // The number of bytes we should truncate.
            var needToTruncate =
                totalSizeBytes - (maxTotalSizeBytes * truncateKeepPercentage).toLong()

            val oldestFirst = allLogs.sortedBy { it.getLastModifiedTime() }

            for (oldest in oldestFirst) {
                if (needToTruncate <= 0) break

                val size = oldest.fileSize()
                if (size <= needToTruncate) {
                    // Size of the the oldest log is less than what we need to truncate so
                    // we can just delete the file. Note that this can never be the current log.
                    oldest.deleteExisting()
                    needToTruncate -= size
                } else {
                    // Size of the oldest log is greater than what we need to truncate so we
                    // have to truncate the file.
                    val tmpFile = logDir.resolve("${oldest.fileName}.tmp")
                    if (!tmpFile.exists()) tmpFile.createFile()

                    val isCurrentLog = oldest == log.logFilePath
                    if (isCurrentLog) log.writer.close()

                    copyNBytesFromEnd(oldest, tmpFile, needToTruncate)
                    Files.move(tmpFile, oldest, StandardCopyOption.REPLACE_EXISTING)

                    if (isCurrentLog) log = FileAndWriter.create(log.logFilePath)
                    needToTruncate = 0
                }
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
