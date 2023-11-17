package net.mullvad.mullvadvpn.dataproxy

import android.content.Context
import java.io.File
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

const val PROBLEM_REPORT_LOGS_FILE = "problem_report.txt"

sealed interface SendProblemReportResult {
    data object Success : SendProblemReportResult

    sealed interface Error : SendProblemReportResult {
        data object CollectLog : Error

        // This is usually due to network error or bad email address
        data object SendReport : Error
    }
}

data class UserReport(val email: String?, val description: String)

class MullvadProblemReport(context: Context, val dispatcher: CoroutineDispatcher = Dispatchers.IO) {

    private val cacheDirectory = File(context.cacheDir.toURI())
    private val logDirectory = File(context.filesDir.toURI())
    private val logsPath = File(logDirectory, PROBLEM_REPORT_LOGS_FILE)

    init {
        System.loadLibrary("mullvad_jni")
    }

    suspend fun collectLogs(): Boolean =
        withContext(dispatcher) {
            // Delete any old report
            deleteLogs()

            collectReport(logDirectory.absolutePath, logsPath.absolutePath)
        }

    suspend fun sendReport(userReport: UserReport): SendProblemReportResult {
        // If report is not collected then, collect it, if it fails then return error
        if (!logsExists() && !collectLogs()) {
            return SendProblemReportResult.Error.CollectLog
        }

        val sentSuccessfully =
            withContext(dispatcher) {
                sendProblemReport(
                    userReport.email ?: "",
                    userReport.description,
                    logsPath.absolutePath,
                    cacheDirectory.absolutePath
                )
            }

        return if (sentSuccessfully) {
            deleteLogs()
            SendProblemReportResult.Success
        } else {
            SendProblemReportResult.Error.SendReport
        }
    }

    suspend fun readLogs(): List<String> {
        if (!logsExists()) {
            collectLogs()
        }

        return if (logsExists()) {
            logsPath.readLines()
        } else {
            listOf("Failed to collect logs for problem report")
        }
    }

    private fun logsExists() = logsPath.exists()

    fun deleteLogs() {
        logsPath.delete()
    }

    // TODO We should remove the external functions from this class and migrate it to the service
    private external fun collectReport(logDirectory: String, logsPath: String): Boolean

    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String,
        cacheDirectory: String
    ): Boolean
}
