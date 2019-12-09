package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.support.v4.app.Fragment

abstract class ServiceAwareFragment : Fragment() {
    lateinit var parentActivity: MainActivity
        private set

    var serviceConnection: ServiceConnection? = null
        private set

    private var subscriptionId: Int? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

        subscriptionId = parentActivity.serviceNotifier.subscribe { connection ->
            configureServiceConnection(connection)
        }
    }

    override fun onDetach() {
        subscriptionId?.let { id ->
            parentActivity.serviceNotifier.unsubscribe(id)
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
