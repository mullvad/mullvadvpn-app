package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.support.v4.app.Fragment
import net.mullvad.mullvadvpn.util.JobTracker

abstract class ServiceAwareFragment : Fragment() {
    val jobTracker = JobTracker()

    lateinit var parentActivity: MainActivity
        private set

    var serviceConnection: ServiceConnection? = null
        private set

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

        parentActivity.serviceNotifier.subscribe(this) { connection ->
            configureServiceConnection(connection)
        }
    }

    override fun onDetach() {
        parentActivity.serviceNotifier.unsubscribe(this)

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
