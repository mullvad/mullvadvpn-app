package net.mullvad.mullvadvpn.utils

import co.touchlab.kermit.Severity
import java.nio.file.Path
import kotlin.io.path.createFile
import kotlin.io.path.fileSize
import kotlin.io.path.listDirectoryEntries
import kotlin.io.path.readLines
import kotlin.io.path.writeText
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.util.FileLogWriter
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith
import org.junit.jupiter.api.io.TempDir

@ExtendWith(TestCoroutineRule::class)
class FileLogWriterTest {
    @Test
    fun `the log file should be created with the correct file name format`(@TempDir tempDir: Path) =
        runTest {
            FileLogWriter(tempDir, scope = this)
            val logFile = tempDir.listDirectoryEntries().first()
            val fileName = logFile.fileName.toString()
            val nameRegex = Regex("app_log_\\d{4}-\\d{2}-\\d{2}.log")
            assertTrue(nameRegex.matches(fileName), "file name was: $fileName")
        }

    @Test
    fun `a log file with the current date should be written to`(@TempDir tempDir: Path) = runTest {
        val log1 = tempDir.resolve("app_log_2025-10-26.txt").createFile()

        val writer = FileLogWriter(tempDir, scope = this, dispatcher = Dispatchers.Main)

        writer.log(Severity.Debug, "test", "test")

        val new = tempDir.listDirectoryEntries().toSet() - setOf(log1)

        assertEquals(0, log1.fileSize())
        assertEquals(1, new.size)
        assertTrue(new.first().fileSize() > 0)
    }

    @Test
    fun `the oldest log should be deleted when the max log file count is reached`(
        @TempDir tempDir: Path
    ) = runTest {
        val log1 = tempDir.resolve("app_log_2025-10-26.txt").createFile()
        // Sleep is needed to ensure the files have different last modified timestamps
        Thread.sleep(10)
        tempDir.resolve("app_log_2025-10-27.txt").createFile()
        Thread.sleep(10)
        tempDir.resolve("app_log_2025-10-28.txt").createFile()

        FileLogWriter(tempDir, maxFileCount = 3, scope = this, dispatcher = Dispatchers.Main)

        val logFiles = tempDir.listDirectoryEntries()

        assertEquals(3, logFiles.size)
        assertFalse(logFiles.contains(log1))
    }

    @Test
    fun `temporary files should be deleted when the file writer is created`(
        @TempDir tempDir: Path
    ) = runTest {
        tempDir.resolve("app_log_2025-10-26.txt").createFile()
        val tmp = tempDir.resolve("app_log_2025-10-26.txt.tmp").createFile()

        FileLogWriter(tempDir, scope = this, dispatcher = Dispatchers.Main)

        val logFiles = tempDir.listDirectoryEntries()

        assertEquals(2, logFiles.size)
        assertFalse(logFiles.contains(tmp))
    }

    @Test
    fun `a single log file larger than the max size should be truncated`(@TempDir tempDir: Path) =
        runTest {
            val maxBytes = 1024L * 10
            val writer =
                FileLogWriter(
                    tempDir,
                    maxTotalSizeBytes = maxBytes,
                    truncateKeepPercentage = 0.5,
                    checkSizeLimitAfter = 20,
                    scope = this,
                    dispatcher = Dispatchers.Main,
                )
            val logFile = tempDir.listDirectoryEntries().first()

            // Write so we are about 60% full
            val writeTo = (maxBytes * 0.6).toInt()
            var writes = 0
            while (logFile.fileSize() < writeTo) {
                writer.log(Severity.Debug, "x", "x")
                writes += 1
            }

            // Write another 60% of the way to the max size to push over the limit
            repeat(writes) { writer.log(Severity.Debug, "z", "z") }

            // Check that the file was truncated
            assertTrue(
                logFile.fileSize() < maxBytes,
                "expected new size ${logFile.fileSize()} to be less than $maxBytes",
            )
            assertTrue(logFile.fileSize() > 0)

            // Check that the file was truncated from the start, not the end, so that the most
            // recent logs are present in the new file
            val lines = logFile.readLines()
            val (z, x) = lines.partition { it.contains("z") }
            assertTrue(z.size > x.size)
        }

    @Test
    fun `multiple log files that are larger than the max size should truncate the oldest log if big enough`(
        @TempDir tempDir: Path
    ) = runTest {
        val maxBytes = 1024L * 10
        val log1 =
            tempDir.resolve("app_log_2025-10-26.txt").createFile().also {
                it.writeText("a".repeat((maxBytes * 0.4).toInt()))
            }
        val log1OrigSize = log1.fileSize()
        Thread.sleep(10)
        val log2 =
            tempDir.resolve("app_log_2025-10-27.txt").createFile().also {
                it.writeText("b".repeat((maxBytes * 0.4).toInt()))
            }

        val writer =
            FileLogWriter(
                tempDir,
                maxTotalSizeBytes = maxBytes,
                checkSizeLimitAfter = 20,
                scope = this,
                dispatcher = Dispatchers.Main,
            )
        val logFile = tempDir.listDirectoryEntries().find { it != log1 && it != log2 }!!

        // Exceed the size limit.
        // The oldest log's size is greater than the amount we need to truncate.
        val writeTo = (maxBytes * 0.3).toInt()
        while (logFile.fileSize() < writeTo) {
            writer.log(Severity.Debug, "x", "x")
        }

        val logFiles = tempDir.listDirectoryEntries()

        assertEquals(3, logFiles.size)
        assertTrue(log1OrigSize > log1.fileSize())
    }

    @Test
    fun `multiple log files that are larger than the max size should remove the oldest log if small enough`(
        @TempDir tempDir: Path
    ) = runTest {
        val maxBytes = 1024L * 10
        val log1 =
            tempDir.resolve("app_log_2025-10-26.txt").createFile().also {
                it.writeText("a".repeat((maxBytes * 0.1).toInt()))
            }
        Thread.sleep(10)
        val log2 =
            tempDir.resolve("app_log_2025-10-27.txt").createFile().also {
                it.writeText("b".repeat((maxBytes * 0.7).toInt()))
            }
        val log2OrigSize = log2.fileSize()

        val writer =
            FileLogWriter(
                tempDir,
                maxTotalSizeBytes = maxBytes,
                checkSizeLimitAfter = 20,
                scope = this,
                dispatcher = Dispatchers.Main,
            )
        val logFile = tempDir.listDirectoryEntries().find { it != log1 && it != log2 }!!

        // Exceed the size limit.
        // The oldest log's size is smaller than the amount we need to truncate.
        val writeTo = (maxBytes * 0.3).toInt()
        while (logFile.fileSize() < writeTo) {
            writer.log(Severity.Debug, "x", "x")
        }

        val logFiles = tempDir.listDirectoryEntries()

        assertEquals(2, logFiles.size)
        assertTrue(logFiles.contains(log2))
        assertFalse(logFiles.contains(log1))
        // Also check that the second oldest log was truncated.
        assertTrue(log2OrigSize > log2.fileSize())
    }
}
