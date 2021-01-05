package net.mullvad.talpid.tunnel

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class ActionAfterDisconnect : Parcelable {
    Nothing, Block, Reconnect
}
