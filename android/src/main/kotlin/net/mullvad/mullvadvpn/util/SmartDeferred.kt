package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.launch
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

class SmartDeferred<T>(private val deferred: Deferred<T>) {
    private val jobTracker = JobTracker()

    fun awaitThen(action: T.() -> Unit): Long? {
        return jobTracker.newJob(GlobalScope.launch(Dispatchers.Default) {
            deferred.await().action()
        })
    }

    fun cancelJob(jobId: Long) {
        jobTracker.cancelJob(jobId)
    }

    fun cancelAllJobs() {
        jobTracker.cancelAllJobs()
    }
}
