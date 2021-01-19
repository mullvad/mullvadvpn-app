package net.mullvad.mullvadvpn.ui

import android.content.Context
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.util.JobTracker

abstract class ServiceAwareFragment : Fragment() {
    val jobTracker = JobTracker()

    open val isSecureScreen = false

    lateinit var parentActivity: MainActivity
        private set

    var serviceConnection: ServiceConnection? = null
        private set

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

        if (isSecureScreen) {
            parentActivity.enterSecureScreen(this)
        }

        parentActivity.serviceNotifier.subscribe(this) { connection ->
            configureServiceConnection(connection)
        }
    }

    override fun onDestroyView() {
        jobTracker.cancelAllJobs()

        super.onDestroyView()
    }

    override fun onDetach() {
        parentActivity.serviceNotifier.unsubscribe(this)

        if (isSecureScreen) {
            parentActivity.leaveSecureScreen(this)
        }

        super.onDetach()
    }

    abstract fun onNewServiceConnection(serviceConnection: ServiceConnection)

    open fun onNoServiceConnection() {
    }

    private fun configureServiceConnection(connection: ServiceConnection?) {
        serviceConnection = connection

        if (connection != null) {
            onNewServiceConnection(connection)
        } else {
            onNoServiceConnection()
        }
    }
}
