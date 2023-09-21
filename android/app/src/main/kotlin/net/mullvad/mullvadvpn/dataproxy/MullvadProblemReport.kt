package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import java.io.File
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

const val PROBLEM_REPORT_FILE = "problem_report.txt"

sealed interface SendProblemReportResult {
    data object Success : SendProblemReportResult

    sealed interface Error : SendProblemReportResult {
        data object CollectLog : Error

        // This is usually due to network error or bad email address
        data object SendReport : Error
    }
}

data class UserReport(val email: String, val message: String)

class MullvadProblemReport(context: Context) {
    private val logDirectory = File(context.filesDir.toURI())
    private val cacheDirectory = File(context.cacheDir.toURI())
    private val problemReportPath = File(logDirectory, PROBLEM_REPORT_FILE)

    private var hasCollectedReport = false

    init {
        System.loadLibrary("mullvad_jni")
    }

    private suspend fun collectReport() =
        withContext(Dispatchers.IO) {
            val logDirectoryPath = logDirectory.absolutePath
            val reportPath = problemReportPath.absolutePath
            // Delete any old report
            deleteReport()
            hasCollectedReport = collectReport(logDirectoryPath, reportPath)
        }

    suspend fun sendReport(userReport: UserReport): SendProblemReportResult {
        if (!hasCollectedReport) {
            collectReport()
        }
        if (!hasCollectedReport) {
            return SendProblemReportResult.Error.CollectLog
        }

        val sentSuccessfully =
            withContext(Dispatchers.IO) {
                sendProblemReport(
                    userReport.email,
                    userReport.message,
                    problemReportPath.absolutePath,
                    cacheDirectory.absolutePath
                )
            }

        return if (sentSuccessfully) {
            deleteReport()
            SendProblemReportResult.Success
        } else {
            SendProblemReportResult.Error.SendReport
        }
    }

    suspend fun readLogs(): List<String> {
        if (!hasCollectedReport) {
            collectReport()
        }

        return if (hasCollectedReport) {
            problemReportPath.readLines()
        } else {
            listOf("Failed to collect logs for problem report")
        }
    }

    fun deleteReport() {
        problemReportPath.delete()
        hasCollectedReport = false
    }

    // TODO We should remove the external functions from this class and migrate it to the service
    private external fun collectReport(logDirectory: String, reportPath: String): Boolean

    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String,
        cacheDirectory: String
    ): Boolean
}
