package net.mullvad.mullvadvpn.ui

import android.content.Context
import net.mullvad.mullvadvpn.ui.fragments.BaseFragment
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.util.JobTracker
import org.koin.android.ext.android.inject

abstract class ServiceAwareFragment : BaseFragment() {
    private val serviceConnectionManager: ServiceConnectionManager by inject()

    val jobTracker = JobTracker()

    open val isSecureScreen = false

    lateinit var parentActivity: MainActivity
        private set

    var serviceConnectionContainer: ServiceConnectionContainer? = null
        private set

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity

        if (isSecureScreen) {
            parentActivity.enterSecureScreen(this)
        }

        serviceConnectionManager.serviceNotifier.subscribe(this) { connection ->
            configureServiceConnection(connection)
        }
    }

    override fun onDestroyView() {
        jobTracker.cancelAllJobs()

        super.onDestroyView()
    }

    override fun onDetach() {
        serviceConnectionManager.serviceNotifier.unsubscribe(this)

        if (isSecureScreen) {
            parentActivity.leaveSecureScreen(this)
        }

        super.onDetach()
    }

    abstract fun onNewServiceConnection(serviceConnectionContainer: ServiceConnectionContainer)

    open fun onNoServiceConnection() {
    }

    private fun configureServiceConnection(
        serviceConnectionContainer: ServiceConnectionContainer?
    ) {
        this.serviceConnectionContainer = serviceConnectionContainer

        if (serviceConnectionContainer != null) {
            onNewServiceConnection(serviceConnectionContainer)
        } else {
            onNoServiceConnection()
        }
    }
}
