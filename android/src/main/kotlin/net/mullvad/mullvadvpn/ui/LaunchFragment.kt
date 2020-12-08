package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnection

class LaunchFragment : ServiceAwareFragment() {
    private val hasAccountToken = CompletableDeferred<Boolean>()

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        serviceConnection.settingsListener.accountNumberNotifier.subscribe(this) { accountToken ->
            hasAccountToken.complete(accountToken != null)
        }
    }

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

    override fun onStart() {
        super.onStart()

        jobTracker.newUiJob("advanceToNextScreen") {
            advanceToNextScreen()
        }
    }

    override fun onStop() {
        jobTracker.cancelJob("advanceToNextScreen")

        super.onStop()
    }

    private suspend fun advanceToNextScreen() {
        if (hasAccountToken.await()) {
            advanceToConnectScreen()
        } else {
            advanceToLoginScreen()
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
