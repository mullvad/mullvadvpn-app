package net.mullvad.mullvadvpn.lib.model

import android.os.Parcel
import android.os.Parcelable
import kotlinx.parcelize.Parceler
import kotlinx.parcelize.Parcelize
import kotlinx.parcelize.TypeParceler

@JvmInline
@Parcelize
@TypeParceler<IntRange, IntRangeParceler>
value class PortRange(val value: IntRange) : Parcelable {
    operator fun contains(port: Port): Boolean = port.value in value

    fun toFormattedString(): String =
        if (value.first == value.last) {
            value.first.toString()
        } else {
            "${value.first}-${value.last}"
        }
}

object IntRangeParceler : Parceler<IntRange> {
    override fun create(parcel: Parcel) = IntRange(parcel.readInt(), parcel.readInt())

    override fun IntRange.write(parcel: Parcel, flags: Int) {
        parcel.writeInt(start)
        parcel.writeInt(endInclusive)
    }
}
