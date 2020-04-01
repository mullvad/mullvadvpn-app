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
import android.widget.Toast
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.KeygenFailure
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.TimeAgoFormatter
import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

val RFC3339_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss.SSSSSSSSSS z")

class WireguardKeyFragment : ServiceDependentFragment(OnNoService.GoToLaunchScreen) {
    private lateinit var timeAgoFormatter: TimeAgoFormatter

    private var currentJob: Job? = null
    private var updateViewsJob: Job? = null
    private var tunnelStateListener: Int? = null
    private var tunnelState: TunnelState = TunnelState.Disconnected()
    private lateinit var urlController: BlockingController
    private var generatingKey = false
    private var validatingKey = false

    private var resetReconnectionExpectedJob: Job? = null
    private var reconnectionExpected = false
        set(value) {
            field = value

            resetReconnectionExpectedJob?.cancel()

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

    private fun resetReconnectionExpected() {
        resetReconnectionExpectedJob = GlobalScope.launch(Dispatchers.Main) {
            delay(20_000)

            if (reconnectionExpected) {
                reconnectionExpected = false
                updateViews()
            }
        }
    }

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

        updateViews()

        return view
    }

    override fun onSafelyResume() {
        tunnelStateListener = connectionProxy.onUiStateChange.subscribe { uiState ->
            synchronized(this@WireguardKeyFragment) {
                tunnelState = uiState

                if (generatingKey) {
                    reconnectionExpected = !(tunnelState is TunnelState.Disconnected)
                } else if (tunnelState is TunnelState.Connected) {
                    reconnectionExpected = false
                }
            }

            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }

        keyStatusListener.onKeyStatusChange = { newKeyStatus ->
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }
    }

    override fun onSafelyPause() {
        tunnelStateListener?.let { listener ->
            connectionProxy.onUiStateChange.unsubscribe(listener)
        }

        keyStatusListener.onKeyStatusChange = null
        currentJob?.cancel()
        updateViewsJob?.cancel()
        resetReconnectionExpectedJob?.cancel()
        validatingKey = false
        generatingKey = false
        urlController.onPause()
    }

    private fun updateViewJob() = GlobalScope.launch(Dispatchers.Main) {
        updateViews()
    }

    private fun updateViews() {
        clearErrorMessage()

        setGenerateButton()
        setVerifyButton()

        when (val keyState = keyStatusListener.keyStatus) {
            null -> {
                publicKey.information = null
            }

            is KeygenEvent.NewKey -> {
                val key = keyState.publicKey
                val publicKeyString = Base64.encodeToString(key.key, Base64.NO_WRAP)
                val publicKeyAge =
                    DateTime.parse(key.dateCreated, RFC3339_FORMAT).withZone(DateTimeZone.UTC)

                publicKey.information = publicKeyString.substring(0, 20) + "..."
                keyAge.information = timeAgoFormatter.format(publicKeyAge)

                keyState.verified?.let { verified ->
                    if (verified) {
                        setStatusMessage(R.string.wireguard_key_valid, R.color.green)
                    } else {
                        setStatusMessage(R.string.wireguard_key_invalid, R.color.red)
                    }
                }

                keyState.replacementFailure?.let { error -> showKeygenFailure(error) }
            }
            else -> {
                showKeygenFailure(keyState.failure())
            }
        }
        drawNoConnectionState()
    }

    private fun setStatusMessage(message: Int, color: Int) {
        statusMessage.setText(message)
        statusMessage.setTextColor(resources.getColor(color))
        statusMessage.visibility = View.VISIBLE
    }

    private fun clearErrorMessage() {
        statusMessage.visibility = View.GONE
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
        generateKeyButton.setEnabled(!generatingKey && !validatingKey)

        if (keyStatusListener.keyStatus is KeygenEvent.NewKey) {
            generateKeyButton.setText(R.string.wireguard_replace_key)
        } else {
            generateKeyButton.setText(R.string.wireguard_generate_key)
        }
    }

    private fun setVerifyButton() {
        val keyState = keyStatusListener.keyStatus

        verifyKeyButton.setEnabled(!generatingKey && !validatingKey && keyState?.failure() == null)
    }

    private fun drawNoConnectionState() {
        manageKeysButton.setEnabled(true)

        when (tunnelState) {
            is TunnelState.Connecting, is TunnelState.Disconnecting -> {
                if (!reconnectionExpected) {
                    setStatusMessage(R.string.wireguard_key_connectivity, R.color.red)
                    generateKeyButton.setEnabled(false)
                }
            }
            is TunnelState.Error -> {
                setStatusMessage(R.string.wireguard_key_blocked_state_message, R.color.red)
                generateKeyButton.setEnabled(false)
                verifyKeyButton.setEnabled(false)
                manageKeysButton.setEnabled(false)
            }
        }
    }

    private fun onGenerateKeyPress() {
        currentJob?.cancel()

        synchronized(this) {
            generatingKey = true
            validatingKey = false
            reconnectionExpected = !(tunnelState is TunnelState.Disconnected)
        }

        updateViews()

        currentJob = GlobalScope.launch(Dispatchers.Main) {
            publicKey.information = null
            keyAge.information = null

            keyStatusListener.generateKey().join()

            generatingKey = false
            updateViews()
        }
    }

    private fun onValidateKeyPress() {
        currentJob?.cancel()
        validatingKey = true
        generatingKey = false
        updateViews()
        currentJob = GlobalScope.launch(Dispatchers.Main) {
            statusMessage.visibility = View.GONE
            verifyingKeySpinner.visibility = View.VISIBLE

            keyStatusListener.verifyKey().join()

            verifyingKeySpinner.visibility = View.GONE
            statusMessage.visibility = View.VISIBLE
            validatingKey = false

            when (val state = keyStatusListener.keyStatus) {
                is KeygenEvent.NewKey -> {
                    if (state.verified == null) {
                        Toast.makeText(parentActivity,
                            R.string.wireguard_key_verification_failure,
                            Toast.LENGTH_SHORT).show()
                    }
                }
            }
            updateViews()
        }
    }
}
