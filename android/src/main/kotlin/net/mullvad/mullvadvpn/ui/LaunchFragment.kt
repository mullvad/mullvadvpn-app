package net.mullvad.mullvadvpn.ui

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.service.MullvadDaemon

class LaunchFragment : ServiceAwareFragment() {
    private val hasAccountToken = CompletableDeferred<Boolean>()

    private var accountTokenCheckJob: Job? = null
    private lateinit var advanceToNextScreenJob: Job

    override fun onNewServiceConnection(serviceConnection: ServiceConnection) {
        accountTokenCheckJob = checkForAccountToken(serviceConnection.daemon)
    }

    override fun onNoServiceConnection() {
        accountTokenCheckJob?.cancel()
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

    override fun onResume() {
        super.onResume()
        advanceToNextScreenJob = advanceToNextScreen()
    }

    override fun onPause() {
        advanceToNextScreenJob.cancel()
        super.onPause()
    }

    override fun onDestroy() {
        accountTokenCheckJob?.cancel()
        super.onDestroy()
    }

    private fun checkForAccountToken(daemon: MullvadDaemon) =
            GlobalScope.async(Dispatchers.Default) {
        val settings = daemon.getSettings()

        hasAccountToken.complete(settings.accountToken != null)
    }

    private fun advanceToNextScreen() = GlobalScope.launch(Dispatchers.Main) {
        if (hasAccountToken.await()) {
            advanceToConnectScreen()
        } else {
            advanceToLoginScreen()
        }
    }

    private fun advanceToLoginScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, LoginFragment())
            commit()
        }
    }

    private fun advanceToConnectScreen() {
        fragmentManager?.beginTransaction()?.apply {
            replace(R.id.main_fragment, ConnectFragment())
            commit()
        }
    }
}
