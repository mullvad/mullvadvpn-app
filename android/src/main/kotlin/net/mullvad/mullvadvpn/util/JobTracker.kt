package net.mullvad.mullvadvpn.util

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class JobTracker {
    private val jobs = HashMap<Long, Job>()
    private val namedJobs = HashMap<String, Long>()

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

    fun newJob(name: String, job: Job): Long {
        synchronized(namedJobs) {
            cancelJob(name)

            val newJobId = newJob(job)

            namedJobs.put(name, newJobId)

            return newJobId
        }
    }

    fun newBackgroundJob(name: String, jobBody: suspend () -> Unit): Long {
        return newJob(name, GlobalScope.launch(Dispatchers.Default) { jobBody() })
    }

    fun newUiJob(name: String, jobBody: suspend () -> Unit): Long {
        return newJob(name, GlobalScope.launch(Dispatchers.Main) { jobBody() })
    }

    fun cancelJob(name: String) {
        synchronized(namedJobs) {
            namedJobs.remove(name)?.let { oldJobId ->
                cancelJob(oldJobId)
            }
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
