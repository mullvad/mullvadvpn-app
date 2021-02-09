package net.mullvad.mullvadvpn.applist

import androidx.recyclerview.widget.DefaultItemAnimator
import androidx.recyclerview.widget.RecyclerView
import net.mullvad.mullvadvpn.model.ListItemData

class ProgressListItemAnimator : DefaultItemAnimator() {

    private val originalRemoveDuration = removeDuration

    override fun animateRemove(holder: RecyclerView.ViewHolder): Boolean {
        if (holder.itemViewType == ListItemData.PROGRESS) {
            removeDuration = 200
        }
        return super.animateRemove(holder)
    }

    override fun onAnimationFinished(viewHolder: RecyclerView.ViewHolder) {
        if (viewHolder.itemViewType == ListItemData.PROGRESS) {
            removeDuration = originalRemoveDuration
        }
        super.onAnimationFinished(viewHolder)
    }
}
