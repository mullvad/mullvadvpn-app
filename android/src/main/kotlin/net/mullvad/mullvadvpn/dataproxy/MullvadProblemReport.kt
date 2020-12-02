package net.mullvad.mullvadvpn.dataproxy

import java.io.File
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.launch

const val PROBLEM_REPORT_FILE = "problem_report.txt"

class MullvadProblemReport {
    val logDirectory = CompletableDeferred<File>()
    val resourcesDirectory = CompletableDeferred<File>()

    private val problemReportPath = GlobalScope.async(Dispatchers.Default) {
        File(logDirectory.await(), PROBLEM_REPORT_FILE)
    }

    private var collectJob: Deferred<Boolean>? = null
    private var sendJob: Deferred<Boolean>? = null
    private var deleteJob: Job? = null

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
                    val logDirectoryPath = logDirectory.await().absolutePath
                    val reportPath = problemReportPath.await().absolutePath

                    deleteReportFile().join()
                    collectReport(logDirectoryPath, reportPath)
                }
            }
        }
    }

    suspend fun load(): String {
        if (collectJob == null) {
            collect()
        }

        if (collectJob?.await() ?: false) {
            return problemReportPath.await().readText()
        } else {
            return "Failed to collect logs for problem report"
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
                            problemReportPath.await().absolutePath,
                            resourcesDirectory.await().absolutePath
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

    fun deleteReportFile(): Job {
        synchronized(this) {
            val oldDeleteJob = deleteJob

            val job = GlobalScope.launch(Dispatchers.Default) {
                oldDeleteJob?.join()
                problemReportPath.await().delete()
                collectJob = null
            }

            deleteJob = job

            return job
        }
    }

    private external fun collectReport(logDirectory: String, reportPath: String): Boolean
    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String,
        resourcesDirectory: String
    ): Boolean
}
