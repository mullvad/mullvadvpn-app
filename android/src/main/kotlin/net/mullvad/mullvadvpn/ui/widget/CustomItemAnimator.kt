package net.mullvad.mullvadvpn.ui.widget

import android.support.v7.widget.DefaultItemAnimator
import android.support.v7.widget.RecyclerView.ViewHolder
import kotlin.math.round

class CustomItemAnimator : DefaultItemAnimator() {
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
            var translationX = view.translationX
            var translationY = view.translationY

            view.animate().setUpdateListener { _ ->
                val deltaX = round(translationX - view.translationX)
                val deltaY = round(translationY - view.translationY)

                onMove?.invoke(deltaX.toInt(), deltaY.toInt())

                translationX -= deltaX
                translationY -= deltaY
            }

            return true
        } else {
            return false
        }
    }
}
