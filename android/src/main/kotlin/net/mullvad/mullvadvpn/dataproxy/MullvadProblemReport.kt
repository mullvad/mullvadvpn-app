package net.mullvad.mullvadvpn.dataproxy

import java.io.File
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async

const val PROBLEM_REPORT_FILE = "problem_report.txt"

class MullvadProblemReport(val logDirectory: File) {
    private val problemReportPath = File(logDirectory, PROBLEM_REPORT_FILE)

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
                    collectReport(logDirectory.absolutePath, problemReportPath.absolutePath)
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
                            sendProblemReport(
                                userEmail,
                                userMessage,
                                problemReportPath.absolutePath
                            )

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
        problemReportPath.delete()
    }

    private external fun collectReport(logDirectory: String, reportPath: String): Boolean
    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String
    ): Boolean
}
