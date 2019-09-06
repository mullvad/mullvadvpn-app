package net.mullvad.mullvadvpn

import kotlinx.coroutines.launch
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.support.v4.app.Fragment
import android.util.Base64
import android.util.Log
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast

import java.util.Locale
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import org.joda.time.DateTime
import org.joda.time.format.DateTimeFormat

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.util.SmartDeferred

val RFC3339_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss.SSSSSSSSSS z")
val KEY_AGE_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm")

class WireguardKeyFragment : Fragment() {
    private var keyState: KeygenEvent? = null
    private var currentJob: Job? = null
    private var updateViewsJob: Job? = null
    private var tunnelStateListener: Int? = null
    private var tunnelStateSubscriptionJob: Long? = null
    private var tunnelState: TunnelState = TunnelState.Disconnected()
    private lateinit var parentActivity: MainActivity
    private lateinit var connectionProxy: SmartDeferred<ConnectionProxy>
    private lateinit var keyStatusListener: KeyStatusListener
    private var generatingKey = false
    private var validatingKey = false

    private lateinit var publicKey: TextView
    private lateinit var publicKeyAgeDisplay: View
    private lateinit var publicKeyAge: TextView
    private lateinit var statusMessage: TextView
    private lateinit var visitWebsiteView: View
    private lateinit var actionButton: Button
    private lateinit var actionSpinner: ProgressBar


    override fun onAttach(context: Context) {
        super.onAttach(context)
        parentActivity = context as MainActivity
        keyStatusListener = parentActivity.keyStatusListener
        connectionProxy = parentActivity.connectionProxy
    }

    override fun onCreateView(
            inflater: LayoutInflater,
            container: ViewGroup?,
            savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.wireguard_key, container, false)

        view.findViewById<View>(R.id.back).setOnClickListener {
            parentActivity.onBackPressed()
        }


        statusMessage = view.findViewById<TextView>(R.id.wireguard_key_status)
        visitWebsiteView = view.findViewById<View>(R.id.wireguard_manage_keys)
        publicKey = view.findViewById<TextView>(R.id.wireguard_public_key)
        publicKeyAgeDisplay = view.findViewById<TextView>(R.id.wireguard_key_age_display)
        publicKeyAge = view.findViewById<TextView>(R.id.wireguard_key_age)
        actionButton = view.findViewById<Button>(R.id.wg_key_button)
        actionSpinner = view.findViewById<ProgressBar>(R.id.wg_action_spinner)

        visitWebsiteView.visibility = View.VISIBLE
        visitWebsiteView.setOnClickListener {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(parentActivity.getString(R.string.account_url)))
            startActivity(intent)
        }

        updateViews()

        return view
    }

    private fun updateViewJob() = GlobalScope.launch(Dispatchers.Main) {
        updateViews()
    }


    private fun updateViews() {
        clearErrorMessage()

        actionButton.setClickable(true)

        publicKeyAgeDisplay.visibility = View.INVISIBLE;

        when (val keyState = keyStatusListener.keyStatus) {
            null -> {
                publicKey.visibility = View.INVISIBLE
                setGenerateButton()
            }
            is KeygenEvent.TooManyKeys -> {

                setStatusMessage(R.string.too_many_keys, R.color.red)
                setGenerateButton()
            }
            is KeygenEvent.GenerationFailure -> {
                setStatusMessage(R.string.failed_to_generate_key, R.color.red)
                setGenerateButton()
            }
            is KeygenEvent.NewKey -> {
                val key = keyState.publicKey;
                val publicKeyString = Base64.encodeToString(key.key, Base64.NO_WRAP)
                publicKey.visibility = View.VISIBLE
                publicKey.setText(publicKeyString)

                publicKeyAgeDisplay.visibility = View.VISIBLE;
                publicKeyAge.setText(formatKeyAge(key.age))

                setVerifyButton()

                if (keyState.verified != null) {
                    if (keyState.verified) {
                        setStatusMessage(R.string.wireguard_key_valid, R.color.green)
                    } else {
                        setStatusMessage(R.string.wireguard_key_invalid, R.color.red)
                        setGenerateButton()
                    }
                }
            }
        }
        drawNoConnectionState()
    }

    private fun setStatusMessage(message: Int, color: Int) {
        statusMessage.setText(message)
        statusMessage.setTextColor(parentActivity.getColor(color))
        statusMessage.visibility = View.VISIBLE
    }

    private fun clearErrorMessage() {
        statusMessage.visibility = View.GONE
    }

    private fun setGenerateButton() {
        if (generatingKey) {
            showActionSpinner()
            return
        }
        actionSpinner.visibility = View.GONE
        actionButton.visibility = View.VISIBLE
        actionButton.setText(R.string.wireguard_generate_key)
        actionButton.setOnClickListener {
            onGenerateKeyPress()
        }
    }

    private fun setVerifyButton() {
        if (validatingKey) {
            showActionSpinner()
            return
        }
        actionSpinner.visibility = View.GONE
        actionButton.visibility = View.VISIBLE
        actionButton.setText(R.string.wireguard_verify_key)
        actionButton.setOnClickListener {
            onValidateKeyPress()
        }
    }

    private fun showActionSpinner() {
        actionButton.visibility = View.GONE
        actionSpinner.visibility = View.VISIBLE
    }

    private fun drawNoConnectionState() {
        when (tunnelState) {
            is TunnelState.Connecting, is TunnelState.Disconnecting -> {
                statusMessage.setText(R.string.wireguard_key_connectivity)
                statusMessage.visibility = View.VISIBLE
                actionButton.visibility = View.GONE
                actionSpinner.visibility = View.VISIBLE
            }
        }
    }

    private fun onGenerateKeyPress() {
        currentJob?.cancel()
        generatingKey = true
        validatingKey = false
        updateViews()
        currentJob = GlobalScope.launch(Dispatchers.Main) {
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
            keyStatusListener.verifyKey().join()
            validatingKey = false
            when (val state = keyStatusListener.keyStatus) {
                is KeygenEvent.NewKey -> {
                    if (state.verified == null) {
                        Toast.makeText(parentActivity, R.string.wireguard_key_verification_failure, Toast.LENGTH_SHORT).show()
                    }
                }
            }
            updateViews()
        }
    }

    override fun onPause() {
        tunnelStateSubscriptionJob?.let { jobId ->
            connectionProxy.cancelJob(jobId)
        }

        tunnelStateListener?.let { listener ->
            connectionProxy.awaitThen {
                onUiStateChange.unsubscribe(listener)
            }
        }

        keyStatusListener.onKeyStatusChange = null
        currentJob?.cancel()
        updateViewsJob?.cancel()
        validatingKey = false
        generatingKey = false
        super.onPause()
    }

    override fun onResume() {
        super.onResume()

        tunnelStateSubscriptionJob = connectionProxy.awaitThen {
            tunnelStateListener = onUiStateChange.subscribe { uiState ->
                tunnelState = uiState
                updateViewsJob?.cancel()
                updateViewsJob = updateViewJob()
            }
        }

        keyStatusListener.onKeyStatusChange = { _ ->
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }
    }

    private fun formatKeyAge(rfc3339: String): String {
        return KEY_AGE_FORMAT.print(DateTime.parse(rfc3339, RFC3339_FORMAT))
    }
}
