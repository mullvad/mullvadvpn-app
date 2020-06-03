package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.content.Intent
import android.graphics.drawable.Drawable
import android.net.Uri
import android.view.View
import android.view.View.MeasureSpec
import android.widget.ImageView
import android.widget.TextView
import kotlin.properties.Delegates.observable
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.AppVersionInfoCache
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import net.mullvad.talpid.tunnel.ParameterGenerationError

class NotificationBanner(
    val parentView: View,
    val context: Context,
    val versionInfoCache: AppVersionInfoCache,
    val daemon: MullvadDaemon
) {
    enum class ExternalLink { Download, KeyManagement }

    private val resources = context.resources

    private val keyManagementUrl = context.getString(R.string.wg_key_url)
    private val downloadUrl = Uri.parse(context.getString(R.string.download_url))

    private val errorImage = resources.getDrawable(R.drawable.icon_notification_error, null)
    private val warningImage = resources.getDrawable(R.drawable.icon_notification_warning, null)

    private val banner: View = parentView.findViewById(R.id.notification_banner)
    private val status: ImageView = parentView.findViewById(R.id.notification_status)
    private val title: TextView = parentView.findViewById(R.id.notification_title)
    private val message: TextView = parentView.findViewById(R.id.notification_message)
    private val icon: View = parentView.findViewById(R.id.notification_icon)

    private var height: Int by observable(0) { _, oldValue, newValue ->
        if (oldValue != newValue) {
            onHeightChange?.invoke(newValue)
        }
    }

    private var updateJob: Job? = null

    private var externalLink: ExternalLink? = null
    private var visible = false

    private val keyManagementController = BlockingController(
        object : BlockableView {
            override fun setEnabled(enabled: Boolean) {
                if (enabled) {
                    banner.setAlpha(1f)
                    banner.setClickable(true)
                } else {
                    banner.setAlpha(0.5f)
                    banner.setClickable(false)
                }
            }

            override fun onClick(): Job {
                return GlobalScope.launch(Dispatchers.Default) {
                    val token = daemon.getWwwAuthToken()
                    val url = Uri.parse(keyManagementUrl + "?token=" + token)
                    context.startActivity(Intent(Intent.ACTION_VIEW, url))
                }
            }
        }
    )

    var onHeightChange by observable<((Int) -> Unit)?>(null) { _, _, newListener ->
        newListener?.invoke(height)
    }

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
        versionInfoCache.onUpdate = {
            updateJob = GlobalScope.launch(Dispatchers.Main) { update() }
        }
    }

    fun onPause() {
        versionInfoCache.onUpdate = null
        updateJob?.cancel()
        keyManagementController.onPause()
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
            is KeygenEvent.TooManyKeys -> {
                        externalLink = ExternalLink.KeyManagement
                        showError(R.string.wireguard_error, R.string.too_many_keys)
            }
            is KeygenEvent.GenerationFailure -> {
                        showError(R.string.wireguard_error, R.string.failed_to_generate_key)
            }
        }

        return true
    }

    private fun updateBasedOnTunnelState(): Boolean {
        val state = tunnelState

        when (state) {
            is TunnelState.Disconnecting -> {
                when (state.actionAfterDisconnect) {
                    ActionAfterDisconnect.Nothing -> return false
                    ActionAfterDisconnect.Block -> showBlocking(null)
                    ActionAfterDisconnect.Reconnect -> showBlocking(null)
                }
            }
            is TunnelState.Disconnected -> return false
            is TunnelState.Connecting -> showBlocking(null)
            is TunnelState.Connected -> return false
            is TunnelState.Error -> showBlocking(state.errorState)
        }

        return true
    }

    private fun updateBasedOnVersionInfo(): Boolean {
        if (!versionInfoCache.isOutdated && versionInfoCache.isSupported) {
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

            externalLink = ExternalLink.Download

            show(statusImage, title, description)
        }

        return true
    }

    private fun showBlocking(errorState: ErrorState?) {
        val cause = errorState?.cause

        val messageText = when (cause) {
            null -> null
            is ErrorStateCause.AuthFailed -> R.string.auth_failed
            is ErrorStateCause.Ipv6Unavailable -> R.string.ipv6_unavailable
            is ErrorStateCause.SetFirewallPolicyError -> R.string.set_firewall_policy_error
            is ErrorStateCause.SetDnsError -> R.string.set_dns_error
            is ErrorStateCause.StartTunnelError -> R.string.start_tunnel_error
            is ErrorStateCause.IsOffline -> R.string.is_offline
            is ErrorStateCause.TapAdapterProblem -> R.string.tap_adapter_problem
            is ErrorStateCause.TunnelParameterError -> {
                when (cause.error) {
                    ParameterGenerationError.NoMatchingRelay -> R.string.no_matching_relay
                    ParameterGenerationError.NoMatchingBridgeRelay -> {
                        R.string.no_matching_bridge_relay
                    }
                    ParameterGenerationError.NoWireguardKey -> R.string.no_wireguard_key
                    ParameterGenerationError.CustomTunnelHostResultionError -> {
                        R.string.custom_tunnel_host_resolution_error
                    }
                }
            }
            is ErrorStateCause.VpnPermissionDenied -> R.string.vpn_permission_denied_error
        }

        // if the error state is null, we can assume that we are secure
        if (errorState?.isBlocking ?: true) {
            showError(R.string.blocking_internet, messageText)
        } else {
            val updatedMessageText = when (cause) {
                is ErrorStateCause.VpnPermissionDenied -> messageText
                else -> R.string.failed_to_block_internet
            }

            showError(R.string.not_blocking_internet, updatedMessageText)
        }
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

        height = recalculateHeight()
    }

    private fun hide() {
        if (visible) {
            visible = false
            banner.animate().translationY(-banner.height.toFloat()).setDuration(350).withEndAction {
                banner.visibility = View.INVISIBLE
            }
        }
    }

    private fun recalculateHeight(): Int {
        banner.apply {
            val widthSpec = MeasureSpec.makeMeasureSpec(measuredWidth, MeasureSpec.AT_MOST)
            val heightSpec = MeasureSpec.makeMeasureSpec(0, MeasureSpec.UNSPECIFIED)

            measure(widthSpec, heightSpec)

            return measuredHeight
        }
    }

    private fun onClick() {
        val externalLink = this.externalLink

        when (externalLink) {
            ExternalLink.Download -> {
                context.startActivity(Intent(Intent.ACTION_VIEW, this.downloadUrl))
            }
            ExternalLink.KeyManagement -> {
                this.keyManagementController.action()
            }
        }
    }
}
