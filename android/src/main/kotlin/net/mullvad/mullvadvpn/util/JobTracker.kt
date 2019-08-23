package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

class JobTracker {
    private val jobs = HashMap<Long, Job>()

    private var jobIdCounter = 0L
    private var active = true

    fun newJob(job: Job): Long? {
        if (active) {
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
        } else {
            job.cancel()

            return null
        }
    }

    fun cancelJob(jobId: Long) {
        synchronized(jobs) {
            jobs.remove(jobId)?.cancel()
        }
    }

    fun cancelAllJobs() {
        active = false

        synchronized(jobs) {
            for (job in jobs.values) {
                job.cancel()
            }

            jobs.clear()
        }
    }
}
