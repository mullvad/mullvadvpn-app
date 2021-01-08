package net.mullvad.mullvadvpn.ui.widget

import androidx.recyclerview.widget.DefaultItemAnimator
import androidx.recyclerview.widget.RecyclerView.LayoutManager
import androidx.recyclerview.widget.RecyclerView.ViewHolder
import kotlin.math.round

class CustomItemAnimator : DefaultItemAnimator() {
    var layoutManager: LayoutManager? = null

    var onMove: ((Int, Int) -> Unit)? = null

    override fun animateMove(
        holder: ViewHolder,
        fromX: Int,
        fromY: Int,
        toX: Int,
        toY: Int
    ): Boolean {
        if (super.animateMove(holder, fromX, fromY, toX, toY)) {
            var view = holder.itemView

            if (view == layoutManager?.getChildAt(0)) {
                var translationX = view.translationX
                var translationY = view.translationY

                view.animate().setUpdateListener { _ ->
                    val deltaX = round(translationX - view.translationX)
                    val deltaY = round(translationY - view.translationY)

                    onMove?.invoke(deltaX.toInt(), deltaY.toInt())

                    translationX -= deltaX
                    translationY -= deltaY
                }
            }

            return true
        } else {
            return false
        }
    }
}
