package net.mullvad.mullvadvpn

import android.content.Context
import android.content.Intent
import android.graphics.drawable.Drawable
import android.net.Uri
import android.widget.ImageView
import android.widget.TextView
import android.view.View

import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.model.BlockReason
import net.mullvad.mullvadvpn.model.ParameterGenerationError
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.KeygenFailure
import net.mullvad.mullvadvpn.model.TunnelState

class NotificationBanner(
    val parentView: View,
    val context: Context,
    val versionInfoCache: AppVersionInfoCache
) {
    private val resources = context.resources

    private val accountUrl = Uri.parse(context.getString(R.string.account_url))
    private val downloadUrl = Uri.parse(context.getString(R.string.download_url))

    private val errorImage = resources.getDrawable(R.drawable.icon_notification_error, null)
    private val warningImage = resources.getDrawable(R.drawable.icon_notification_warning, null)

    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private val status: ImageView = parentView.findViewById(R.id.notification_status)
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

    fun onResume() {
        versionInfoCache.onUpdate = { update() }
    }

    fun onPause() {
        versionInfoCache.onUpdate = null
    }

    private fun update() {
        externalLink = null
        updateBasedOnTunnelState() || updateBasedOnKeyState() || updateBasedOnVersionInfo()
    }

    private fun updateBasedOnKeyState(): Boolean {
        val keyState = keyState
        when (keyState) {
            null -> return false
            is KeygenEvent.NewKey -> return false
            is KeygenEvent.Failure -> {
                when (keyState.failure) {
                    is KeygenFailure.TooManyKeys -> {
                        externalLink = accountUrl
                        showError(R.string.wireguard_error, R.string.too_many_keys)
                    }
                    is KeygenFailure.GenerationFailure -> {
                        showError(R.string.wireguard_error, R.string.failed_to_generate_key)
                    }
                }
            }
        }

        return true
    }

    private fun updateBasedOnTunnelState(): Boolean {
        val state = tunnelState

        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    is ActionAfterDisconnect.Nothing -> return false
                    is ActionAfterDisconnect.Block -> showBlocking(null)
                    is ActionAfterDisconnect.Reconnect -> showBlocking(null)
                }
            }
            is TunnelState.Disconnected -> return false
            is TunnelState.Connecting -> showBlocking(null)
            is TunnelState.Connected -> return false
            is TunnelState.Blocked -> showBlocking(state.reason)
        }

        return true
    }

    private fun updateBasedOnVersionInfo(): Boolean {
        if (versionInfoCache.isLatest) {
            hide()
        } else {
            val title: Int
            val statusImage: Drawable
            val template: Int

            if (versionInfoCache.isSupported) {
                title = R.string.update_available
                template = R.string.update_available_description
                statusImage = warningImage
            } else {
                title = R.string.unsupported_version
                template = R.string.unsupported_version_description
                statusImage = errorImage
            }

            val parameter = versionInfoCache.upgradeVersion
            val description = context.getString(template, parameter)

            externalLink = downloadUrl

            show(statusImage, title, description)
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
            is BlockReason.IsOffline -> R.string.is_offline
            is BlockReason.TapAdapterProblem -> R.string.tap_adapter_problem
            is BlockReason.ParameterGeneration -> {
                when (reason.error) {
                    is ParameterGenerationError.NoMatchingRelay -> R.string.no_matching_relay
                    is ParameterGenerationError.NoMatchingBridgeRelay -> R.string.no_matching_bridge_relay
                    is ParameterGenerationError.NoWireguardKey -> R.string.no_wireguard_key
                    is ParameterGenerationError.CustomTunnelHostResultionError -> R.string.custom_tunnel_host_resolution_error
                }
            }
        }
        showError(R.string.blocking_internet, messageText)
    }

    private fun showError(titleText: Int, messageText: Int?) {
        showError(titleText, messageText?.let { context.getString(it) })
    }

    private fun showError(titleText: Int, messageText: String?) {
        show(errorImage, titleText, messageText)
    }

    private fun show(statusImage: Drawable, titleText: Int, messageText: String?) {
        if (!visible) {
            visible = true
            banner.visibility = View.VISIBLE
            banner.translationY = -banner.height.toFloat()
            banner.animate().translationY(0.0F).setDuration(350).start()
        }

        status.setImageDrawable(statusImage)
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
