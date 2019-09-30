package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class JobTracker {
    private val jobs = HashMap<Long, Job>()

    private var jobIdCounter = 0L

    fun newJob(job: Job): Long {
        synchronized(jobs) {
            val jobId = jobIdCounter

            jobIdCounter += 1

            jobs.put(jobId, GlobalScope.launch(Dispatchers.Default) {
                job.join()

                synchronized(jobs) {
                    jobs.remove(jobId)
                }
            })

            return jobId
        }
    }

    fun cancelJob(jobId: Long) {
        synchronized(jobs) {
            jobs.remove(jobId)?.cancel()
        }
    }

    fun cancelAllJobs() {
        synchronized(jobs) {
            for (job in jobs.values) {
                job.cancel()
            }

            jobs.clear()
        }
    }
}
