package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.os.Bundle
import android.util.Base64
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.delay
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.KeygenFailure
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.ui.widget.Button
import net.mullvad.mullvadvpn.ui.widget.CopyableInformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView
import net.mullvad.mullvadvpn.ui.widget.InformationView.WhenMissing
import net.mullvad.mullvadvpn.ui.widget.UrlButton
import net.mullvad.mullvadvpn.util.TimeAgoFormatter
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

val RFC3339_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss.SSSSSSSSSS z")

class WireguardKeyFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    override val isSecureScreen = true

    sealed class ActionState {
        class Idle(val verified: Boolean) : ActionState()
        class Generating(val replacing: Boolean) : ActionState()
        class Verifying() : ActionState()
    }

    private lateinit var timeAgoFormatter: TimeAgoFormatter
    private lateinit var titleController: CollapsibleTitleController

    private var greenColor: Int = 0
    private var redColor: Int = 0

    private var actionCompletion: CompletableDeferred<Unit>? = null
    private var tunnelState: TunnelState = TunnelState.Disconnected

    private var actionState: ActionState = ActionState.Idle(false)
        set(value) {
            if (field != value) {
                field = value
                updateKeySpinners()
                updateStatusMessage()
                updateGenerateKeyButtonState()
                updateGenerateKeyButtonText()
                updateVerifyKeyButtonState()
                updateVerifyingKeySpinner()
            }
        }

    private var keyStatus: KeygenEvent? = null
        set(value) {
            if (field != value) {
                field = value
                updateKeyInformation()
                updateStatusMessage()
                updateGenerateKeyButtonText()
                updateVerifyKeyButtonState()

                actionCompletion?.complete(Unit)
            }
        }

    private var isOffline = true
        set(value) {
            if (field != value) {
                field = value
                updateStatusMessage()
                updateGenerateKeyButtonState()
                updateVerifyKeyButtonState()
                manageKeysButton.setEnabled(!value)
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
    private lateinit var manageKeysButton: UrlButton
    private lateinit var generateKeyButton: Button
    private lateinit var verifyKeyButton: Button

    override fun onAttach(context: Context) {
        super.onAttach(context)

        redColor = context.getColor(R.color.red)
        greenColor = context.getColor(R.color.green)
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
        publicKey = view.findViewById(R.id.public_key)
        keyAge = view.findViewById(R.id.key_age)

        generateKeyButton = view.findViewById<Button>(R.id.generate_key).apply {
            setOnClickAction("action", jobTracker) {
                onGenerateKeyPress()
            }
        }

        verifyKeyButton = view.findViewById<Button>(R.id.verify_key).apply {
            setOnClickAction("action", jobTracker) {
                onValidateKeyPress()
            }
        }

        verifyingKeySpinner = view.findViewById(R.id.verifying_key_spinner)

        manageKeysButton = view.findViewById<UrlButton>(R.id.manage_keys).apply {
            prepare(authTokenCache, jobTracker)
        }

        titleController = CollapsibleTitleController(view)

        return view
    }

    override fun onSafelyStart() {
        connectionProxy.onUiStateChange.subscribe(this) { uiState ->
            jobTracker.newUiJob("tunnelStateUpdate") {
                tunnelState = uiState

                if (actionState is ActionState.Generating) {
                    reconnectionExpected = !(tunnelState is TunnelState.Disconnected)
                } else if (tunnelState is TunnelState.Connected) {
                    reconnectionExpected = false
                }

                isOffline = uiState is TunnelState.Error &&
                    uiState.errorState.cause is ErrorStateCause.IsOffline
            }
        }

        keyStatusListener.onKeyStatusChange.subscribe(this) { newKeyStatus ->
            jobTracker.newUiJob("keyStatusUpdate") {
                keyStatus = newKeyStatus
            }
        }

        actionState = ActionState.Idle(false)
    }

    override fun onSafelyStop() {
        connectionProxy.onUiStateChange.unsubscribe(this)
        keyStatusListener.onKeyStatusChange.unsubscribe(this)

        if (!(actionState is ActionState.Idle)) {
            actionState = ActionState.Idle(false)
        }
    }

    override fun onSafelyDestroyView() {
        titleController.onDestroy()
    }

    private fun updateKeySpinners() {
        when (actionState) {
            is ActionState.Generating -> {
                publicKey.whenMissing = WhenMissing.ShowSpinner
                keyAge.whenMissing = WhenMissing.ShowSpinner
            }
            is ActionState.Verifying, is ActionState.Idle -> {
                publicKey.whenMissing = WhenMissing.Nothing
                keyAge.whenMissing = WhenMissing.Nothing
            }
        }
    }

    private fun updateKeyInformation() {
        when (val keyState = keyStatus) {
            is KeygenEvent.NewKey -> {
                val key = keyState.publicKey
                val publicKeyString = Base64.encodeToString(key.key, Base64.NO_WRAP)
                val publicKeyAge =
                    DateTime.parse(key.dateCreated, RFC3339_FORMAT).withZone(DateTimeZone.UTC)

                publicKey.error = null
                publicKey.information = publicKeyString
                keyAge.information = timeAgoFormatter.format(publicKeyAge)
            }
            is KeygenEvent.TooManyKeys, is KeygenEvent.GenerationFailure -> {
                publicKey.error = resources.getString(failureMessage(keyState.failure()!!))
                publicKey.information = null
                keyAge.information = null
            }
            null -> {
                publicKey.error = null
                publicKey.information = null
                keyAge.information = null
            }
        }
    }

    private fun updateStatusMessage() {
        when (val state = actionState) {
            is ActionState.Generating -> statusMessage.visibility = View.INVISIBLE
            is ActionState.Verifying -> statusMessage.visibility = View.INVISIBLE
            is ActionState.Idle -> {
                if (!isOffline) {
                    updateKeyStatus(state.verified, keyStatus)
                } else {
                    updateOfflineStatus()
                }
            }
        }
    }

    private fun updateOfflineStatus() {
        if (reconnectionExpected) {
            setStatusMessage(R.string.wireguard_key_reconnecting, greenColor)
        }
    }

    private fun updateKeyStatus(verificationWasDone: Boolean, keyStatus: KeygenEvent?) {
        if (keyStatus is KeygenEvent.NewKey) {
            val replacementFailure = keyStatus.replacementFailure

            if (replacementFailure != null) {
                setStatusMessage(failureMessage(replacementFailure), redColor)
            } else {
                updateKeyIsValid(verificationWasDone, keyStatus.verified)
            }
        } else {
            statusMessage.visibility = View.INVISIBLE
        }
    }

    private fun updateKeyIsValid(verificationWasDone: Boolean, verified: Boolean?) {
        when (verified) {
            true -> setStatusMessage(R.string.wireguard_key_valid, greenColor)
            false -> setStatusMessage(R.string.wireguard_key_invalid, redColor)
            null -> {
                if (verificationWasDone) {
                    setStatusMessage(R.string.wireguard_key_verification_failure, redColor)
                } else {
                    statusMessage.visibility = View.INVISIBLE
                }
            }
        }
    }

    private fun updateGenerateKeyButtonState() {
        generateKeyButton.setEnabled(actionState is ActionState.Idle && !isOffline)
    }

    private fun updateGenerateKeyButtonText() {
        val state = actionState
        val replacingKey = state is ActionState.Generating && state.replacing
        val hasKey = keyStatus is KeygenEvent.NewKey

        if (hasKey || replacingKey) {
            generateKeyButton.setText(R.string.wireguard_replace_key)
        } else {
            generateKeyButton.setText(R.string.wireguard_generate_key)
        }
    }

    private fun updateVerifyKeyButtonState() {
        val isIdle = actionState is ActionState.Idle
        val hasKey = keyStatus is KeygenEvent.NewKey

        verifyKeyButton.setEnabled(isIdle && hasKey && !isOffline)
    }

    private fun updateVerifyingKeySpinner() {
        verifyingKeySpinner.visibility = when (actionState) {
            is ActionState.Verifying -> View.VISIBLE
            else -> View.INVISIBLE
        }
    }

    private fun setStatusMessage(message: Int, color: Int) {
        statusMessage.setText(message)
        statusMessage.setTextColor(color)
        statusMessage.visibility = View.VISIBLE
    }

    private fun failureMessage(failure: KeygenFailure): Int {
        when (failure) {
            KeygenFailure.TooManyKeys -> return R.string.too_many_keys
            KeygenFailure.GenerationFailure -> return R.string.failed_to_generate_key
        }
    }

    private suspend fun onGenerateKeyPress() {
        actionState = ActionState.Generating(keyStatus is KeygenEvent.NewKey)
        reconnectionExpected = !(tunnelState is TunnelState.Disconnected)

        keyStatus = null

        actionCompletion = CompletableDeferred()
        keyStatusListener.generateKey()
        actionCompletion?.await()

        actionState = ActionState.Idle(false)
    }

    private suspend fun onValidateKeyPress() {
        actionState = ActionState.Verifying()

        actionCompletion = CompletableDeferred()
        keyStatusListener.verifyKey()
        actionCompletion?.await()

        actionState = ActionState.Idle(true)
    }

    private fun resetReconnectionExpected() {
        jobTracker.newUiJob("resetReconnectionExpected") {
            delay(20_000)

            if (reconnectionExpected) {
                reconnectionExpected = false
            }
        }
    }
}
