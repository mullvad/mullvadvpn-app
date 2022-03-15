package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class RemoveDeviceResult : Parcelable {
    Ok,
    NotFound,
    RpcError,
    OtherError
}
