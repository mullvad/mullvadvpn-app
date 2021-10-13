package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch

class SmartDeferred<T>(private val deferred: Deferred<T>) {
    private val jobTracker = JobTracker()

    private var active = true

    fun awaitThen(action: T.() -> Unit): Long? {
        if (active) {
            return jobTracker.newJob(
                GlobalScope.launch(Dispatchers.Default) {
                    deferred.await().action()
                }
            )
        } else {
            return null
        }
    }

    fun cancelJob(jobId: Long) {
        jobTracker.cancelJob(jobId)
    }

    fun cancel() {
        active = false
        jobTracker.cancelAllJobs()
    }
}
