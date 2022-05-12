package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onEach
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection

class LaunchFragment : ServiceAwareFragment() {

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.launch, container, false)

        view.findViewById<View>(R.id.settings).setOnClickListener {
            parentActivity.openSettings()
        }

        return view
    }

    override fun onStop() {
        jobTracker.cancelJob("advanceToNextScreen")
        super.onStop()
    }

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        advanceToNextScreen(serviceConnection.deviceRepository)
    }

    private fun advanceToNextScreen(deviceRepository: DeviceRepository) {
        jobTracker.newUiJob("advanceToNextScreen") {
            deviceRepository.deviceState
                .onEach { state ->
                    if (state.isInitialState()) deviceRepository.refreshDeviceState()
                }
                .first { state -> state.isInitialState().not() }
                .let { deviceState ->
                    when (deviceState) {
                        is DeviceState.LoggedIn -> advanceToConnectScreen()
                        is DeviceState.LoggedOut -> advanceToLoginScreen()
                        else -> Unit
                    }
                }
        }
    }

    private fun advanceToLoginScreen() {
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun advanceToConnectScreen() {
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }
}
