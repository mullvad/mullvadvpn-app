package net.mullvad.mullvadvpn

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.TextView
import android.view.View

import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.BlockReason
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState

class NotificationBanner(val parentView: View, val context: Context) {
    private val accountUrl = Uri.parse(context.getString(R.string.account_url))

    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private val title: TextView = parentView.findViewById(R.id.notification_title)
    private val message: TextView = parentView.findViewById(R.id.notification_message)
    private val icon: View = parentView.findViewById(R.id.notification_icon)

    private var externalLink: Uri? = null
    private var visible = false

    var keyState: KeygenEvent? = null
        set(value) {
            field = value
            update()
        }

    var tunnelState: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value
            update()
        }

    init {
        banner.setOnClickListener { onClick() }
    }

    private fun update() {
        externalLink = null
        updateBasedOnKeyState() || updateBasedOnTunnelState()
    }

    private fun updateBasedOnKeyState(): Boolean {
        when (keyState) {
            null -> return false
            is KeygenEvent.NewKey -> return false
            is KeygenEvent.TooManyKeys -> {
                externalLink = accountUrl
                show(R.string.wireguard_error, R.string.too_many_keys)
            }
            is KeygenEvent.GenerationFailure -> {
                show(R.string.wireguard_error, R.string.failed_to_generate_key)
            }
        }

        return true
    }

    private fun updateBasedOnTunnelState(): Boolean {
        val state = tunnelState

        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    is ActionAfterDisconnect.Nothing -> hide()
                    is ActionAfterDisconnect.Block -> showBlocking(null)
                    is ActionAfterDisconnect.Reconnect -> showBlocking(null)
                }
            }
            is TunnelState.Disconnected -> hide()
            is TunnelState.Connecting -> showBlocking(null)
            is TunnelState.Connected -> hide()
            is TunnelState.Blocked -> showBlocking(state.reason)
        }

        return true
    }

    private fun showBlocking(reason: BlockReason?) {
        val messageText = when (reason) {
            null -> null
            is BlockReason.AuthFailed -> R.string.auth_failed
            is BlockReason.Ipv6Unavailable -> R.string.ipv6_unavailable
            is BlockReason.SetFirewallPolicyError -> R.string.set_firewall_policy_error
            is BlockReason.SetDnsError -> R.string.set_dns_error
            is BlockReason.StartTunnelError -> R.string.start_tunnel_error
            is BlockReason.NoMatchingRelay -> R.string.no_matching_relay
            is BlockReason.IsOffline -> R.string.is_offline
            is BlockReason.TapAdapterProblem -> R.string.tap_adapter_problem
        }

        show(R.string.blocking_internet, messageText)
    }

    private fun show(titleText: Int, messageText: Int?) {
        if (!visible) {
            visible = true
            banner.visibility = View.VISIBLE
            banner.translationY = -banner.height.toFloat()
            banner.animate().translationY(0.0F).setDuration(350).start()
        }

        title.setText(titleText)

        if (messageText == null) {
            message.visibility = View.GONE
        } else {
            message.setText(messageText)
            message.visibility = View.VISIBLE
        }

        if (externalLink == null) {
            banner.setClickable(false)
            icon.visibility = View.GONE
        } else {
            banner.setClickable(true)
            icon.visibility = View.VISIBLE
        }
    }

    private fun hide() {
        if (visible) {
            visible = false
            banner.animate().translationY(-banner.height.toFloat()).setDuration(350).withEndAction {
                banner.visibility = View.INVISIBLE
            }
        }
    }

    private fun onClick() {
        val externalLink = this.externalLink

        if (externalLink != null) {
            context.startActivity(Intent(Intent.ACTION_VIEW, externalLink))
        }
    }
}
