package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.os.Handler
import android.support.v4.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button

import net.mullvad.mullvadvpn.model.TunnelStateTransition

class ConnectFragment : Fragment() {
    private lateinit var actionButton: ConnectActionButton
    private lateinit var headerBar: HeaderBar
    private lateinit var notificationBanner: NotificationBanner
    private lateinit var status: ConnectionStatus

    private lateinit var parentActivity: MainActivity

    private var daemon = CompletableDeferred<MullvadDaemon>()
    private var vpnPermission = CompletableDeferred<Unit>()

    private var generateWireguardKeyJob = generateWireguardKey()

    private var activeAction: Job? = null
    private var attachListenerJob: Job? = null
    private var updateViewJob: Job? = null
    private var waitForDaemonJob: Job? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        parentActivity = context as MainActivity
        waitForDaemonJob = waitForDaemon(parentActivity.asyncDaemon)
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.connect, container, false)

        view.findViewById<Button>(R.id.switch_location).setOnClickListener {
            openSwitchLocationScreen()
        }

        headerBar = HeaderBar(view, context!!)
        notificationBanner = NotificationBanner(view)
        status = ConnectionStatus(view, context!!)

        actionButton = ConnectActionButton(view)
        actionButton.apply {
            onConnect = { connect() }
            onCancel = { disconnect() }
            onDisconnect = { disconnect() }
        }

        attachListenerJob = attachListener()

        return view
    }

    override fun onDestroyView() {
        waitForDaemonJob?.cancel()
        attachListenerJob?.cancel()
        detachListener()
        generateWireguardKeyJob.cancel()
        updateViewJob?.cancel()
        super.onDestroyView()
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, resultData: Intent?) {
        if (resultCode == Activity.RESULT_OK) {
            vpnPermission.complete(Unit)
        }
    }

    private fun waitForDaemon(asyncDaemon: Deferred<MullvadDaemon>) =
            GlobalScope.launch(Dispatchers.Default) {
        daemon.complete(asyncDaemon.await())
    }

    private fun attachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = { state -> updateViewJob = updateView(state) }
    }

    private fun detachListener() = GlobalScope.launch(Dispatchers.Default) {
        daemon.await().onTunnelStateChange = null
    }

    private fun generateWireguardKey() = GlobalScope.launch(Dispatchers.Default) {
        val daemon = this@ConnectFragment.daemon.await()
        val key = daemon.getWireguardKey()

        if (key == null) {
            daemon.generateWireguardKey()
        }
    }

    private fun requestVpnPermission() {
        val intent = VpnService.prepare(parentActivity)

        vpnPermission = CompletableDeferred<Unit>()

        if (intent != null) {
            startActivityForResult(intent, 0)
        } else {
            onActivityResult(0, Activity.RESULT_OK, null)
        }
    }

    private fun connect() {
        updateViewToPreConnecting()
        activeAction?.cancel()

        requestVpnPermission()

        activeAction = GlobalScope.launch(Dispatchers.Default) {
            vpnPermission.await()
            generateWireguardKeyJob.join()
            daemon.await().connect()
        }
    }

    private fun disconnect() {
        activeAction?.cancel()

        activeAction = GlobalScope.launch(Dispatchers.Default) {
            daemon.await().disconnect()
        }
    }

    private fun updateViewToPreConnecting() {
        val connecting = TunnelStateTransition.Connecting()
        val disconnected = TunnelStateTransition.Disconnected()

        headerBar.setState(disconnected)

        actionButton.state = connecting
        notificationBanner.setState(connecting)
        status.setState(connecting)
    }

    private fun updateView(state: TunnelStateTransition) = GlobalScope.launch(Dispatchers.Main) {
        actionButton.state = state
        headerBar.setState(state)
        notificationBanner.setState(state)
        status.setState(state)
    }

    private fun openSwitchLocationScreen() {
        fragmentManager?.beginTransaction()?.apply {
            setCustomAnimations(
                R.anim.fragment_enter_from_bottom,
                R.anim.do_nothing,
                R.anim.do_nothing,
                R.anim.fragment_exit_to_bottom
            )
            replace(R.id.main_fragment, SelectLocationFragment())
            addToBackStack(null)
            commit()
        }
    }
}
