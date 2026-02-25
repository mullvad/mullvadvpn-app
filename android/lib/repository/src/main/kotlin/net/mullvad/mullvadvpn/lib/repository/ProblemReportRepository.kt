@file:JvmName("ProblemReportRepositoryKt")

package net.mullvad.mullvadvpn.lib.repository

import android.content.Context
import java.io.File
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointFromIntentHolder
import net.mullvad.mullvadvpn.lib.endpoint.ApiEndpointOverride
import net.mullvad.mullvadvpn.lib.model.UserReport
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus

const val PROBLEM_REPORT_LOGS_FILE = "problem_report.txt"

sealed interface SendProblemReportResult {
    data object Success : SendProblemReportResult

    sealed interface Error : SendProblemReportResult {
        data object CollectLog : Error

        // This is usually due to network error or bad email address
        data object SendReport : Error
    }
}

class ProblemReportRepository(
    context: Context,
    private val apiEndpointOverride: ApiEndpointOverride?,
    private val apiEndpointFromIntentHolder: ApiEndpointFromIntentHolder,
    private val accountRepository: AccountRepository,
    kermitFileLogDirName: String,
    private val paymentLogic: PaymentLogic,
    val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    init {
        System.loadLibrary("mullvad_jni")
    }

    private val _problemReport = MutableStateFlow(UserReport("", ""))
    val problemReport: StateFlow<UserReport> = _problemReport.asStateFlow()
    private val cacheDirectory = File(context.cacheDir.toURI())
    private val logDirectory = File(context.filesDir.toURI())
    private val problemReportOutputPath = File(logDirectory, PROBLEM_REPORT_LOGS_FILE)
    private val kermitFileLogDirPath = File(logDirectory, kermitFileLogDirName)
    private val collectReportMutex = Mutex()

    fun setEmail(email: String) = _problemReport.update { it.copy(email = email) }

    fun setDescription(description: String) =
        _problemReport.update { it.copy(description = description) }

    suspend fun collectLogs(): Boolean =
        withContext(dispatcher) {

            // Lock to avoid potential truncation of the log file that the daemon creates
            collectReportMutex.withLock {
                // Delete any old report
                deleteLogs()

                val availableProducts = paymentLogic.allAvailableProducts()

                collectReport(
                    logDirectory = logDirectory.absolutePath,
                    kermitFileLogDir = kermitFileLogDirPath.absolutePath,
                    problemReportOutputPath = problemReportOutputPath.absolutePath,
                    unverifiedPurchases =
                        availableProducts?.count {
                            it.status == PaymentStatus.VERIFICATION_IN_PROGRESS
                        } ?: 0,
                    pendingPurchases =
                        availableProducts?.count { it.status == PaymentStatus.PENDING } ?: 0,
                )
            }
        }

    suspend fun sendReport(
        userReport: UserReport,
        includeAccountId: Boolean,
    ): SendProblemReportResult {
        // If report is not collected then, collect it, if it fails then return error
        if (!logsExists() && !collectLogs()) {
            return SendProblemReportResult.Error.CollectLog
        }

        val sentSuccessfully =
            withContext(dispatcher) {
                val intentApiOverride = apiEndpointFromIntentHolder.apiEndpointOverride
                val apiOverride =
                    if (BuildConfig.DEBUG && intentApiOverride != null) {
                        intentApiOverride
                    } else {
                        apiEndpointOverride
                    }

                sendProblemReport(
                    userEmail = userReport.email ?: "",
                    userMessage = userReport.description,
                    accountId =
                        if (includeAccountId) {
                            accountRepository.accountData.value?.id?.value?.toString()
                        } else {
                            null
                        },
                    reportPath = problemReportOutputPath.absolutePath,
                    cacheDirectory = cacheDirectory.absolutePath,
                    apiEndpointOverride = apiOverride,
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
            problemReportOutputPath.readLines()
        } else {
            listOf("Failed to collect logs for problem report")
        }
    }

    private fun logsExists() = problemReportOutputPath.exists()

    fun deleteLogs() {
        problemReportOutputPath.delete()
    }

    // TODO We should remove the external functions from this class and migrate it to the service
    private external fun collectReport(
        logDirectory: String,
        kermitFileLogDir: String,
        problemReportOutputPath: String,
        unverifiedPurchases: Int,
        pendingPurchases: Int,
    ): Boolean

    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        accountId: String?,
        reportPath: String,
        cacheDirectory: String,
        apiEndpointOverride: ApiEndpointOverride?,
    ): Boolean
}
