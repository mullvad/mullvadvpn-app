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
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast

import net.mullvad.mullvadvpn.dataproxy.ConnectionProxy
import net.mullvad.mullvadvpn.dataproxy.KeyStatusListener
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class WireguardKeyFragment : Fragment() {
    private var TAG = "keyfragment";
    private var keyState: KeygenEvent? = null;
    private var currentJob: Job? = null;
    private var updateViewsJob: Job? = null;
    private lateinit var parentActivity: MainActivity
    private lateinit var connectionProxy: ConnectionProxy
    private lateinit var keyStatusListener: KeyStatusListener
    private var generatingKey = false
    private var validatingKey = false

    private lateinit var publicKey: TextView
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
        visitWebsiteView = view.findViewById<View>(R.id.wireguard_key_visit_website)
        publicKey = view.findViewById<TextView>(R.id.wireguard_public_key)
        actionButton = view.findViewById<Button>(R.id.wg_key_button)
        actionSpinner = view.findViewById<ProgressBar>(R.id.wg_action_spinner)

        updateViews()

        return view
    }

    private fun updateViewJob() = GlobalScope.launch(Dispatchers.Main) {
        updateViews()
    }


    private fun updateViews() {
        clearErrorMessage()
        visitWebsiteView.visibility = View.GONE

        actionButton.setClickable(true)

        when (val keyState = keyStatusListener.keyStatus) {
            null -> {
                publicKey.visibility = View.INVISIBLE
                setGenerateButton()
            }
            is KeygenEvent.TooManyKeys -> {
                visitWebsiteView.visibility = View.VISIBLE
                visitWebsiteView.setOnClickListener {
                    val intent = Intent(Intent.ACTION_VIEW, Uri.parse(parentActivity.getString(R.string.account_url)))
                    startActivity(intent)
                }

                setStatusMessage(R.string.too_many_keys, R.color.red)
                setGenerateButton()
            }
            is KeygenEvent.GenerationFailure -> {
                setStatusMessage(R.string.failed_to_generate_key, R.color.red)
                setGenerateButton()
            }
            is KeygenEvent.NewKey -> {
                val publicKeyString = Base64.encodeToString(keyState.publicKey.key, Base64.DEFAULT)
                publicKey.visibility = View.VISIBLE
                publicKey.setText(publicKeyString)

                setValidateButton()

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
            return;
        }
        actionSpinner.visibility = View.GONE
        actionButton.visibility = View.VISIBLE
        actionButton.setText(R.string.wireguard_generate_key)
        actionButton.setOnClickListener {
            onGenerateKeyPress()
        }
    }

    private fun setValidateButton() {
        if (validatingKey) {
            showActionSpinner()
            return;
        }
        actionSpinner.visibility = View.GONE
        actionButton.visibility = View.VISIBLE
        actionButton.setText(R.string.wireguard_validate_key)
        actionButton.setOnClickListener {
            onValidateKeyPress()
        }
    }

    private fun showActionSpinner() {
        actionButton.visibility = View.GONE
        actionSpinner.visibility = View.VISIBLE
    }

    private fun drawNoConnectionState() {
        when (connectionProxy.state) {
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
        generatingKey = true;
        validatingKey = false;
        updateViews()
        currentJob = GlobalScope.launch(Dispatchers.Main) {
            keyStatusListener.generateKey().join()
            generatingKey = false;
            updateViews()
        }
    }

    private fun onValidateKeyPress() {
        currentJob?.cancel()
        validatingKey = true;
        generatingKey = false;
        updateViews()
        currentJob = GlobalScope.launch(Dispatchers.Main) {
            keyStatusListener.verifyKey().join()
            validatingKey = false;
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
        connectionProxy.onUiStateChange = null
        keyStatusListener.onKeyStatusChange = null
        currentJob?.cancel()
        updateViewsJob?.cancel()
        validatingKey = false;
        generatingKey = false;
        super.onPause()
    }

    override fun onResume() {
        super.onResume()
        connectionProxy.onUiStateChange = { _ ->
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }

        keyStatusListener.onKeyStatusChange = { _ ->
            updateViewsJob?.cancel()
            updateViewsJob = updateViewJob()
        }
    }
}
