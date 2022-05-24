package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.flowWithLifecycle
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionContainer
import org.koin.android.ext.android.inject

class LaunchFragment : ServiceAwareFragment() {
    private val deviceRepository: DeviceRepository by inject()

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

    override fun onNewServiceConnection(serviceConnectionContainer: ServiceConnectionContainer) {
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        lifecycleScope.launch {
            deviceRepository.deviceState
                .flowWithLifecycle(lifecycle, Lifecycle.State.RESUMED)
                .collect { deviceState ->
                    when (deviceState) {
                        is DeviceState.LoggedIn -> advanceToConnectScreen()
                        is DeviceState.LoggedOut -> advanceToLoginScreen()
                        is DeviceState.Revoked -> advanceToRevokedScreen()
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

    private fun advanceToRevokedScreen() {
        // TODO: Open revoked screen.
        parentFragmentManager.beginTransaction().apply {
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }
}
