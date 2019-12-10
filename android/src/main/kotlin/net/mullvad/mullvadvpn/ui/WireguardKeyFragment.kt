package net.mullvad.mullvadvpn.ui

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.support.v4.app.Fragment
import android.util.Base64
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import java.util.TimeZone
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.dataproxy.WwwAuthTokenRetriever
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.KeygenFailure
import net.mullvad.mullvadvpn.model.TunnelState
import org.joda.time.DateTime
import org.joda.time.DateTimeZone
import org.joda.time.format.DateTimeFormat

val RFC3339_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm:ss.SSSSSSSSSS z")
val KEY_AGE_FORMAT = DateTimeFormat.forPattern("YYYY-MM-dd HH:mm")

class WireguardKeyFragment : Fragment() {
    private var currentJob: Job? = null
    private var updateViewsJob: Job? = null
    private var tunnelStateListener: Int? = null
    private var tunnelState: TunnelState = TunnelState.Disconnected()
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var keyStatusListener: KeyStatusListener
    private lateinit var parentActivity: MainActivity
    private lateinit var wwwTokenRetriever: WwwAuthTokenRetriever
    private lateinit var urlController: BlockingController
    private var generatingKey = false
    private var validatingKey = false

    private lateinit var publicKey: TextView
    private lateinit var publicKeyAge: TextView
    private lateinit var statusMessage: TextView
    private lateinit var visitWebsiteView: View
    private lateinit var generateButton: Button
    private lateinit var generateSpinner: ProgressBar
    private lateinit var verifyButton: Button
    private lateinit var verifySpinner: ProgressBar

    override fun onAttach(context: Context) {
        super.onAttach(context)
        parentActivity = context as MainActivity
        keyStatusListener = parentActivity.keyStatusListener
        connectionProxy = parentActivity.connectionProxy
        wwwTokenRetriever = parentActivity.wwwAuthTokenRetriever
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
        generateButton = view.findViewById<Button>(R.id.wg_generate_key_button)
        generateSpinner = view.findViewById<ProgressBar>(R.id.wg_generate_key_spinner)
        verifyButton = view.findViewById<Button>(R.id.wg_verify_key_button)
        verifySpinner = view.findViewById<ProgressBar>(R.id.wg_verify_key_spinner)
        publicKeyAge = view.findViewById<TextView>(R.id.wireguard_key_age)

        visitWebsiteView.visibility = View.VISIBLE
        val keyUrl = parentActivity.getString(R.string.wg_key_url)

        urlController = BlockingController(
            object : BlockableView {
                override fun setEnabled(enabled: Boolean) {
                    if (!enabled || tunnelState is TunnelState.Error) {
                        visitWebsiteView.setClickable(false)
                        visitWebsiteView.setAlpha(0.5f)
                    } else {
                        visitWebsiteView.setClickable(true)
                        visitWebsiteView.setAlpha(1f)
                    }
                }

                override fun onClick(): Job {
                    return GlobalScope.launch(Dispatchers.Default) {
                        val token = wwwTokenRetriever.getAuthToken()
                        val intent = Intent(Intent.ACTION_VIEW,
                                            Uri.parse(keyUrl + "?token=" + token))
                        startActivity(intent)
                    }
                }
            }
        )
        visitWebsiteView.setOnClickListener {
            urlController.action()
        }

        updateViews()

        return view
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
                publicKey.visibility = View.INVISIBLE
            }

            is KeygenEvent.NewKey -> {
                val key = keyState.publicKey
                val publicKeyString = Base64.encodeToString(key.key, Base64.NO_WRAP)
                publicKey.visibility = View.VISIBLE
                publicKey.setText(publicKeyString.substring(0, 20) + "...")

                publicKey.setOnClickListener {
                    val label = parentActivity.getString(R.string.wireguard_key_copied_to_clibpoard)
                    val clipboard = parentActivity
                        .getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
                    clipboard.setPrimaryClip(ClipData.newPlainText(label, publicKeyString))

                    Toast.makeText(parentActivity, label, Toast.LENGTH_SHORT)
                        .show()
                }

                publicKeyAge.setText(formatKeyDateCreated(key.dateCreated))

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
        generateButton.setClickable(true)
        generateButton.setAlpha(1f)
        if (validatingKey) {
            generateButton.setClickable(false)
            generateButton.setAlpha(0.5f)
            return
        }
        if (generatingKey) {
            generateButton.visibility = View.GONE
            generateSpinner.visibility = View.VISIBLE
            return
        }
        generateSpinner.visibility = View.GONE
        generateButton.visibility = View.VISIBLE
        if (keyStatusListener.keyStatus is KeygenEvent.NewKey) {
            generateButton.setText(R.string.wireguard_replace_key)
        } else {
            generateButton.setText(R.string.wireguard_generate_key)
        }

        generateButton.setOnClickListener {
            onGenerateKeyPress()
        }
    }

    private fun setVerifyButton() {
        verifyButton.setClickable(true)
        verifyButton.setAlpha(1f)
        val keyState = keyStatusListener.keyStatus
        if (generatingKey || keyState?.failure() != null) {
            verifyButton.setClickable(false)
            verifyButton.setAlpha(0.5f)
            return
        }
        if (validatingKey) {
            verifyButton.visibility = View.GONE
            verifySpinner.visibility = View.VISIBLE
            return
        }
        verifySpinner.visibility = View.GONE
        verifyButton.visibility = View.VISIBLE
        verifyButton.setText(R.string.wireguard_verify_key)
        verifyButton.setOnClickListener {
            onValidateKeyPress()
        }
    }

    private fun drawNoConnectionState() {
        visitWebsiteView.setClickable(true)
        visitWebsiteView.setAlpha(1f)

        when (tunnelState) {
            is TunnelState.Connecting, is TunnelState.Disconnecting -> {
                statusMessage.setText(R.string.wireguard_key_connectivity)
                statusMessage.visibility = View.VISIBLE
                generateButton.visibility = View.GONE
                generateSpinner.visibility = View.VISIBLE
                verifyButton.visibility = View.GONE
                verifySpinner.visibility = View.VISIBLE
            }
            is TunnelState.Error -> {
                statusMessage.setText(R.string.wireguard_key_blocked_state_message)
                statusMessage.visibility = View.VISIBLE
                generateButton.setClickable(false)
                generateButton.setAlpha(0.5f)
                verifyButton.setClickable(false)
                verifyButton.setAlpha(0.5f)
                visitWebsiteView.setClickable(false)
                visitWebsiteView.setAlpha(0.5f)
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
                        Toast.makeText(parentActivity,
                            R.string.wireguard_key_verification_failure,
                            Toast.LENGTH_SHORT).show()
                    }
                }
            }
            updateViews()
        }
    }

    override fun onPause() {
        tunnelStateListener?.let { listener ->
            connectionProxy.onUiStateChange.unsubscribe(listener)
        }

        keyStatusListener.onKeyStatusChange = null
        currentJob?.cancel()
        updateViewsJob?.cancel()
        validatingKey = false
        generatingKey = false
        urlController.onPause()
        super.onPause()
    }

    override fun onResume() {
        super.onResume()

        tunnelStateListener = connectionProxy.onUiStateChange.subscribe { uiState ->
            tunnelState = uiState
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }

        keyStatusListener.onKeyStatusChange = { _ ->
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }
    }

    private fun formatKeyDateCreated(rfc3339: String): String {
        val dateCreated = DateTime.parse(rfc3339, RFC3339_FORMAT).withZone(DateTimeZone.UTC)
        val localTimezone = DateTimeZone.forTimeZone(TimeZone.getDefault())
        return parentActivity.getString(R.string.wireguard_key_age) +
            " " +
            KEY_AGE_FORMAT.print(dateCreated.withZone(localTimezone))
    }
}
