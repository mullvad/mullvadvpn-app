package net.mullvad.mullvadvpn.ui

import android.content.Context
import android.graphics.Rect
import android.support.v7.widget.RecyclerView
import android.support.v7.widget.RecyclerView.ItemDecoration
import android.support.v7.widget.RecyclerView.State
import android.view.View
import kotlin.properties.Delegates.observable

class ListItemDividerDecoration(context: Context) : ItemDecoration() {
    private var bottomOffset = 0
    private var topOffset = 0

    var bottomOffsetId by observable<Int?>(null) { _, _, id ->
        if (id != null) {
            bottomOffset = context.resources.getDimensionPixelSize(id)
        } else {
            bottomOffset = 0
        }
    }

    var topOffsetId by observable<Int?>(null) { _, _, id ->
        if (id != null) {
            topOffset = context.resources.getDimensionPixelSize(id)
        } else {
            topOffset = 0
        }
    }

    override fun getItemOffsets(offsets: Rect, view: View, parent: RecyclerView, state: State) {
        offsets.bottom = bottomOffset
        offsets.top = topOffset
    }
}
