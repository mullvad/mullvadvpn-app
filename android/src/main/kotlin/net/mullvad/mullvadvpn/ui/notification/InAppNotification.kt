package net.mullvad.mullvadvpn.ui.notification

import net.mullvad.mullvadvpn.util.ChangeMonitor
import net.mullvad.mullvadvpn.util.JobTracker

abstract class InAppNotification {
    private val changeMonitor = ChangeMonitor()
    protected val jobTracker = JobTracker()

    var controller: InAppNotificationController? = null

    var status by changeMonitor.monitor(StatusLevel.Error)
        protected set

    var title by changeMonitor.monitor("")
        protected set

    var message by changeMonitor.monitor<String?>(null)
        protected set

    var onClick by changeMonitor.monitor<(suspend () -> Unit)?>(null)
        protected set

    var showIcon by changeMonitor.monitor(false)
        protected set

    var shouldShow by changeMonitor.monitor(false)
        protected set

    open fun onResume() {}
    open fun onPause() {}

    open fun onDestroy() {
        jobTracker.cancelAllJobs()
    }

    protected fun update() {
        val controller = this.controller

        if (controller != null && changeMonitor.changed) {
            controller.notificationChanged(this@InAppNotification)
            changeMonitor.reset()
        }
    }
}
