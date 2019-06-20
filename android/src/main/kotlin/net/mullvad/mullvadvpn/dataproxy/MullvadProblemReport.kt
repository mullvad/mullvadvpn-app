package net.mullvad.mullvadvpn.dataproxy

import java.io.File

import kotlinx.coroutines.async
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope

const val PROBLEM_REPORT_PATH = "/data/data/net.mullvad.mullvadvpn/problem_report.txt"

class MullvadProblemReport {
    private var collectJob: Deferred<Boolean>? = null

    var userEmail = ""
    var userMessage = ""

    init {
        System.loadLibrary("mullvad_jni")
    }

    fun collect() {
        synchronized(this) {
            val currentJob = collectJob

            if (currentJob == null || currentJob.isCompleted) {
                collectJob = GlobalScope.async(Dispatchers.Default) {
                    deleteReportFile()
                    collectReport(PROBLEM_REPORT_PATH)
                }
            }
        }
    }

    private fun deleteReportFile() {
        File(PROBLEM_REPORT_PATH).delete()
    }

    private external fun collectReport(reportPath: String): Boolean
}
