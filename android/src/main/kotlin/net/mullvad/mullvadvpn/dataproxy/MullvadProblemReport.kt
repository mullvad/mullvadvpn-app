package net.mullvad.mullvadvpn.dataproxy

import java.io.File

import kotlinx.coroutines.async
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

const val PROBLEM_REPORT_PATH = "/data/data/net.mullvad.mullvadvpn/problem_report.txt"

class MullvadProblemReport {
    private var collectJob: Deferred<Boolean>? = null
    private var sendJob: Deferred<Boolean>? = null

    var confirmNoEmail: CompletableDeferred<Boolean>? = null

    var userEmail = ""
    var userMessage = ""

    val isActive: Boolean
        get() {
            synchronized(this) {
                val collectJob = this.collectJob
                val sendJob = this.sendJob

                return (collectJob != null && collectJob.isActive) ||
                    (sendJob != null && sendJob.isActive)
            }
        }

    init {
        System.loadLibrary("mullvad_jni")
    }

    fun collect() {
        synchronized(this) {
            if (!isActive) {
                collectJob = GlobalScope.async(Dispatchers.Default) {
                    deleteReportFile()
                    collectReport(PROBLEM_REPORT_PATH)
                }
            }
        }
    }

    fun send(): Deferred<Boolean> {
        synchronized(this) {
            var currentJob = sendJob

            if (currentJob == null || currentJob.isCompleted) {
                currentJob = GlobalScope.async(Dispatchers.Default) {
                    val result = (collectJob?.await() ?: false) &&
                            sendProblemReport(userEmail, userMessage, PROBLEM_REPORT_PATH)

                    if (result) {
                        deleteReportFile()
                    }

                    result
                }

                sendJob = currentJob
            }

            return currentJob
        }
    }

    fun deleteReportFile() {
        File(PROBLEM_REPORT_PATH).delete()
    }

    private external fun collectReport(reportPath: String): Boolean
    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String
    ): Boolean
}
