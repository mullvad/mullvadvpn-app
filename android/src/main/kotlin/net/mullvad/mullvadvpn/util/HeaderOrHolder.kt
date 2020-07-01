package net.mullvad.mullvadvpn.util

import android.support.v7.widget.RecyclerView.ViewHolder
import android.view.View

sealed class HeaderOrHolder<H : ViewHolder>(itemView: View) : ViewHolder(itemView) {
    class Header<H : ViewHolder>(headerView: View) : HeaderOrHolder<H>(headerView)
    class Holder<H : ViewHolder>(val holder: H) : HeaderOrHolder<H>(holder.itemView)
}
