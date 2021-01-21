package net.mullvad.mullvadvpn.service

import android.os.Bundle
import android.os.Message
import android.os.Parcelable
import java.util.ArrayList
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.model.AppVersionInfo as AppVersionInfoData
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.KeygenEvent
import net.mullvad.mullvadvpn.model.LoginStatus as LoginStatusData
import net.mullvad.mullvadvpn.model.RelayList
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.model.VoucherSubmissionResult as VoucherSubmissionResultData

sealed class Event : Parcelable {
    @Parcelize
    class AccountHistory(val history: ArrayList<String>?) : Event(), Parcelable

    @Parcelize
    class AppVersionInfo(val versionInfo: AppVersionInfoData?) : Event(), Parcelable

    @Parcelize
    class AuthToken(val token: String?) : Event(), Parcelable

    @Parcelize
    class CurrentVersion(val version: String?) : Event(), Parcelable

    @Parcelize
    class ListenerReady : Event(), Parcelable

    @Parcelize
    class LoginStatus(val status: LoginStatusData?) : Event(), Parcelable

    @Parcelize
    class NewLocation(val location: GeoIpLocation?) : Event(), Parcelable

    @Parcelize
    class NewRelayList(val relayList: RelayList?) : Event(), Parcelable

    @Parcelize
    class SettingsUpdate(val settings: Settings?) : Event(), Parcelable

    @Parcelize
    class SplitTunnelingUpdate(val excludedApps: ArrayList<String>?) : Event(), Parcelable

    @Parcelize
    class TunnelStateChange(val tunnelState: TunnelState) : Event(), Parcelable

    @Parcelize
    class VoucherSubmissionResult(
        val voucher: String,
        val result: VoucherSubmissionResultData
    ) : Event(), Parcelable

    @Parcelize
    class WireGuardKeyStatus(val keyStatus: KeygenEvent?) : Event(), Parcelable

    val message: Message
        get() = Message.obtain().also { message ->
            message.what = EVENT_MESSAGE
            message.data = Bundle()
            message.data.putParcelable(EVENT_KEY, this)
        }

    companion object {
        const val EVENT_MESSAGE = 1
        const val EVENT_KEY = "event"

        fun fromMessage(message: Message): Event {
            val data = message.data

            data.classLoader = Event::class.java.classLoader

            return data.getParcelable(EVENT_KEY)!!
        }
    }
}
