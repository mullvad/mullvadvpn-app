package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.util.Base64
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.KeygenFailure
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.util.JobTracker
import net.mullvad.mullvadvpn.util.TimeAgoFormatter
import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

val RFC3339_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss.SSSSSSSSSS z")

class WireguardKeyFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    sealed class ActionState {
        class Idle(val verified: Boolean) : ActionState()
        class Generating() : ActionState()
        class Verifying() : ActionState()
    }

    private val jobTracker = JobTracker()

    private lateinit var timeAgoFormatter: TimeAgoFormatter

    private var tunnelStateListener: Int? = null
    private var tunnelState: TunnelState = TunnelState.Disconnected()
    private lateinit var urlController: BlockingController

    private var actionState: ActionState = ActionState.Idle(false)
        set(value) {
            if (field != value) {
                field = value
                updateKeyInformation()
                updateStatus()
                updateButtons()
            }
        }

    private var keyStatus: KeygenEvent? = null
        set(value) {
            if (field != value) {
                field = value
                updateKeyInformation()
                updateStatus()
            }
        }

    private var hasConnectivity = true
        set(value) {
            if (field != value) {
                field = value
                updateStatus()
                updateButtons()
            }
        }

    private var reconnectionExpected = false
        set(value) {
            field = value

            jobTracker.cancelJob("resetReconnectionExpected")

            if (value == true) {
                resetReconnectionExpected()
            }
        }

    private lateinit var publicKey: CopyableInformationView
    private lateinit var keyAge: InformationView
    private lateinit var statusMessage: TextView
    private lateinit var verifyingKeySpinner: View
    private lateinit var manageKeysButton: Button
    private lateinit var generateKeyButton: android.widget.Button
    private lateinit var verifyKeyButton: Button

    override fun onAttach(context: Context) {
        super.onAttach(context)

        timeAgoFormatter = TimeAgoFormatter(context.resources)
    }

    override fun onSafelyCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.wireguard_key, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }

        statusMessage = view.findViewById<TextView>(R.id.wireguard_key_status)
        manageKeysButton = view.findViewById(R.id.manage_keys)
        publicKey = view.findViewById(R.id.public_key)
        keyAge = view.findViewById(R.id.key_age)

        generateKeyButton = view.findViewById<Button>(R.id.generate_key).apply {
            setOnClickListener {
                onGenerateKeyPress()
            }
        }

        verifyKeyButton = view.findViewById<Button>(R.id.verify_key).apply {
            setOnClickListener {
                onValidateKeyPress()
            }
        }

        verifyingKeySpinner = view.findViewById(R.id.verifying_key_spinner)

        val keyUrl = parentActivity.getString(R.string.wg_key_url)

        urlController = BlockingController(
            object : BlockableView {
                override fun setEnabled(enabled: Boolean) {
                    manageKeysButton.setEnabled(enabled && !(tunnelState is TunnelState.Error))
                }

                override fun onClick(): Job {
                    return GlobalScope.launch(Dispatchers.Default) {
                        val token = daemon.getWwwAuthToken()
                        val intent = Intent(Intent.ACTION_VIEW,
                                            Uri.parse(keyUrl + "?token=" + token))
                        startActivity(intent)
                    }
                }
            }
        )
        manageKeysButton.setOnClickListener {
            urlController.action()
        }

        return view
    }

    override fun onSafelyResume() {
        tunnelStateListener = connectionProxy.onUiStateChange.subscribe { uiState ->
            synchronized(this@WireguardKeyFragment) {
                tunnelState = uiState

                if (actionState is ActionState.Generating) {
                    reconnectionExpected = !(tunnelState is TunnelState.Disconnected)
                } else if (tunnelState is TunnelState.Connected) {
                    reconnectionExpected = false
                }

                hasConnectivity = uiState is TunnelState.Connected ||
                    uiState is TunnelState.Disconnected ||
                    (uiState is TunnelState.Error && !uiState.errorState.isBlocking)
            }
        }

        keyStatusListener.onKeyStatusChange = { newKeyStatus ->
            keyStatus = newKeyStatus
        }
    }

    override fun onSafelyPause() {
        tunnelStateListener?.let { listener ->
            connectionProxy.onUiStateChange.unsubscribe(listener)
        }

        if (!(actionState is ActionState.Idle)) {
            actionState = ActionState.Idle(false)
        }

        keyStatusListener.onKeyStatusChange = null
        urlController.onPause()
        jobTracker.cancelAllJobs()
    }

    private fun updateKeyInformation() {
        jobTracker.newUiJob("updateKeyInformation") {
            when (val keyState = keyStatus) {
                is KeygenEvent.NewKey -> {
                    val key = keyState.publicKey
                    val publicKeyString = Base64.encodeToString(key.key, Base64.NO_WRAP)
                    val publicKeyAge =
                        DateTime.parse(key.dateCreated, RFC3339_FORMAT).withZone(DateTimeZone.UTC)

                    publicKey.information = publicKeyString.substring(0, 20) + "..."
                    keyAge.information = timeAgoFormatter.format(publicKeyAge)
                }
                null -> {
                    publicKey.information = null
                    keyAge.information = null
                }
            }
        }
    }

    private fun updateStatus() {
        jobTracker.newUiJob("updateStatus") {
            verifyingKeySpinner.visibility = when (actionState) {
                is ActionState.Verifying -> View.VISIBLE
                else -> View.GONE
            }

            when (val state = actionState) {
                is ActionState.Generating -> statusMessage.visibility = View.GONE
                is ActionState.Verifying -> statusMessage.visibility = View.GONE
                is ActionState.Idle -> {
                    if (hasConnectivity) {
                        updateKeyStatus(state.verified, keyStatus)
                    } else {
                        updateOfflineStatus()
                    }
                }
            }
        }
    }

    private fun updateOfflineStatus() {
        if (reconnectionExpected) {
            setStatusMessage(R.string.wireguard_key_reconnecting, R.color.green)
        } else {
            setStatusMessage(R.string.wireguard_key_blocked_state_message, R.color.red)
        }
    }

    private fun updateKeyStatus(verificationWasDone: Boolean, keyStatus: KeygenEvent?) {
        if (keyStatus is KeygenEvent.NewKey) {
            if (keyStatus.replacementFailure != null) {
                showKeygenFailure(keyStatus.replacementFailure)
            } else {
                updateKeyIsValid(verificationWasDone, keyStatus.verified)
            }
        } else {
            statusMessage.visibility = View.GONE
        }
    }

    private fun updateKeyIsValid(verificationWasDone: Boolean, verified: Boolean?) {
        when (verified) {
            true -> setStatusMessage(R.string.wireguard_key_valid, R.color.green)
            false -> setStatusMessage(R.string.wireguard_key_invalid, R.color.red)
            null -> {
                if (verificationWasDone) {
                    setStatusMessage(R.string.wireguard_key_verification_failure, R.color.red)
                } else {
                    statusMessage.visibility = View.GONE
                }
            }
        }
    }

    private fun updateButtons() {
        jobTracker.newUiJob("updateButtons") {
            val isIdle = actionState is ActionState.Idle

            generateKeyButton.setEnabled(isIdle && hasConnectivity)
            verifyKeyButton.setEnabled(isIdle && hasConnectivity)
            manageKeysButton.setEnabled(hasConnectivity)
        }
    }

    private fun setStatusMessage(message: Int, color: Int) {
        statusMessage.setText(message)
        statusMessage.setTextColor(resources.getColor(color))
        statusMessage.visibility = View.VISIBLE
    }

    private fun showKeygenFailure(failure: KeygenFailure?) {
        when (failure) {
            is KeygenFailure.TooManyKeys -> {
                setStatusMessage(R.string.too_many_keys, R.color.red)
            }
            is KeygenFailure.GenerationFailure -> {
                setStatusMessage(R.string.failed_to_generate_key, R.color.red)
            }
        }
    }

    private fun setGenerateButton() {
        if (keyStatus is KeygenEvent.NewKey) {
            generateKeyButton.setText(R.string.wireguard_replace_key)
        } else {
            generateKeyButton.setText(R.string.wireguard_generate_key)
        }
    }

    private fun onGenerateKeyPress() {
        jobTracker.newBackgroundJob("action") {
            synchronized(this) {
                actionState = ActionState.Generating()
                reconnectionExpected = !(tunnelState is TunnelState.Disconnected)
            }

            keyStatus = null
            keyStatusListener.generateKey().join()

            actionState = ActionState.Idle(false)
        }
    }

    private fun onValidateKeyPress() {
        jobTracker.newBackgroundJob("action") {
            actionState = ActionState.Verifying()
            keyStatusListener.verifyKey().join()
            actionState = ActionState.Idle(true)
        }
    }

    private fun resetReconnectionExpected() {
        jobTracker.newBackgroundJob("resetReconnectionExpected") {
            delay(20_000)

            if (reconnectionExpected) {
                reconnectionExpected = false
            }
        }
    }
}
