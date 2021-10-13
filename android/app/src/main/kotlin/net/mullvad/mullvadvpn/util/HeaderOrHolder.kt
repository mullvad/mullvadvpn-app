package net.mullvad.mullvadvpn.util

import android.view.View
import androidx.recyclerview.widget.RecyclerView.ViewHolder

sealed class HeaderOrHolder<H : ViewHolder>(itemView: View) : ViewHolder(itemView) {
    class Header<H : ViewHolder>(headerView: View) : HeaderOrHolder<H>(headerView)
    class Holder<H : ViewHolder>(val holder: H) : HeaderOrHolder<H>(holder.itemView)
}
