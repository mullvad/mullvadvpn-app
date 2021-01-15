package net.mullvad.mullvadvpn.service

import android.os.Bundle
import android.os.Message
import java.util.ArrayList
import net.mullvad.mullvadvpn.model.AppVersionInfo as AppVersionInfoData
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LoginStatus as LoginStatusData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult as VoucherSubmissionResultData

sealed class Event {
    abstract val type: Type

    val message: Message
        get() = Message.obtain().apply {
            what = type.ordinal
            data = Bundle()

            prepareData(data)
        }

    open fun prepareData(data: Bundle) {}

    class AccountHistory(val history: ArrayList<String>?) : Event() {
        companion object {
            private val historyKey = "history"

            fun buildHistory(data: Bundle): ArrayList<String>? {
                return data.getStringArray(historyKey)?.let { historyArray ->
                    ArrayList(historyArray.toList())
                }
            }
        }

        override val type = Type.AccountHistory

        constructor(data: Bundle) : this(buildHistory(data)) {}

        override fun prepareData(data: Bundle) {
            data.putStringArray(historyKey, history?.toTypedArray())
        }
    }

    class AppVersionInfo(val versionInfo: AppVersionInfoData?) : Event() {
        companion object {
            private val versionInfoKey = "versionInfo"
        }

        override val type = Type.AppVersionInfo

        constructor(data: Bundle) : this(data.getParcelable(versionInfoKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(versionInfoKey, versionInfo)
        }
    }

    class AuthToken(val token: String?) : Event() {
        companion object {
            private val tokenKey = "token"
        }

        override val type = Type.AuthToken

        constructor(data: Bundle) : this(data.getString(tokenKey)) {}

        override fun prepareData(data: Bundle) {
            data.putString(tokenKey, token)
        }
    }

    class CurrentVersion(val version: String?) : Event() {
        companion object {
            private val versionKey = "version"
        }

        override val type = Type.CurrentVersion

        constructor(data: Bundle) : this(data.getString(versionKey)) {}

        override fun prepareData(data: Bundle) {
            data.putString(versionKey, version)
        }
    }

    class ListenerReady : Event() {
        override val type = Type.ListenerReady
    }

    class LoginStatus(val status: LoginStatusData?) : Event() {
        companion object {
            private val statusKey = "status"
        }

        override val type = Type.LoginStatus

        constructor(data: Bundle) : this(data.getParcelable(statusKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(statusKey, status)
        }
    }

    class NewLocation(val location: GeoIpLocation?) : Event() {
        companion object {
            private val locationKey = "location"
        }

        override val type = Type.NewLocation

        constructor(data: Bundle) : this(data.getParcelable(locationKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(locationKey, location)
        }
    }

    class NewRelayList(val relayList: RelayList?) : Event() {
        companion object {
            private val relayListKey = "relayList"
        }

        override val type = Type.NewRelayList

        constructor(data: Bundle) : this(data.getParcelable(relayListKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(relayListKey, relayList)
        }
    }

    class SettingsUpdate(val settings: Settings?) : Event() {
        companion object {
            private val settingsKey = "settings"
        }

        override val type = Type.SettingsUpdate

        constructor(data: Bundle) : this(data.getParcelable(settingsKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(settingsKey, settings)
        }
    }

    class SplitTunnelingUpdate(val excludedApps: ArrayList<String>?) : Event() {
        companion object {
            private val excludedAppsKey = "excludedApps"

            fun buildExcludedApps(data: Bundle): ArrayList<String>? {
                return data.getStringArray(excludedAppsKey)?.let { excludedAppsArray ->
                    ArrayList(excludedAppsArray.toList())
                }
            }
        }

        override val type = Type.SplitTunnelingUpdate

        constructor(data: Bundle) : this(buildExcludedApps(data)) {}

        override fun prepareData(data: Bundle) {
            data.putStringArray(excludedAppsKey, excludedApps?.toTypedArray())
        }
    }

    class TunnelStateChange(val tunnelState: TunnelState) : Event() {
        companion object {
            private val tunnelStateKey = "tunnelState"

            fun buildTunnelState(data: Bundle): TunnelState {
                return data.getParcelable(tunnelStateKey) ?: TunnelState.Disconnected()
            }
        }

        override val type = Type.TunnelStateChange

        constructor(data: Bundle) : this(buildTunnelState(data)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(tunnelStateKey, tunnelState)
        }
    }

    class VoucherSubmissionResult(
        val voucher: String,
        val result: VoucherSubmissionResultData
    ) : Event() {
        companion object {
            private val voucherKey = "voucher"
            private val resultKey = "result"
        }

        override val type = Type.VoucherSubmissionResult

        constructor(data: Bundle) : this(
            data.getString(voucherKey) ?: "",
            data.getParcelable(resultKey) ?: VoucherSubmissionResultData.OtherError()
        ) {}

        override fun prepareData(data: Bundle) {
            data.putString(voucherKey, voucher)
            data.putParcelable(resultKey, result)
        }
    }

    class WireGuardKeyStatus(val keyStatus: KeygenEvent?) : Event() {
        companion object {
            private val keyStatusKey = "keyStatus"
        }

        override val type = Type.WireGuardKeyStatus

        constructor(data: Bundle) : this(data.getParcelable(keyStatusKey)) {}

        override fun prepareData(data: Bundle) {
            data.putParcelable(keyStatusKey, keyStatus)
        }
    }

    enum class Type(val build: (Bundle) -> Event) {
        AccountHistory({ data -> AccountHistory(data) }),
        AppVersionInfo({ data -> AppVersionInfo(data) }),
        AuthToken({ data -> AuthToken(data) }),
        CurrentVersion({ data -> CurrentVersion(data) }),
        ListenerReady({ _ -> ListenerReady() }),
        LoginStatus({ data -> LoginStatus(data) }),
        NewLocation({ data -> NewLocation(data) }),
        NewRelayList({ data -> NewRelayList(data) }),
        SettingsUpdate({ data -> SettingsUpdate(data) }),
        SplitTunnelingUpdate({ data -> SplitTunnelingUpdate(data) }),
        TunnelStateChange({ data -> TunnelStateChange(data) }),
        VoucherSubmissionResult({ data -> VoucherSubmissionResult(data) }),
        WireGuardKeyStatus({ data -> WireGuardKeyStatus(data) }),
    }

    companion object {
        fun fromMessage(message: Message): Event {
            val type = Type.values()[message.what]

            return type.build(message.data)
        }
    }
}
