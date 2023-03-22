package net.mullvad.mullvadvpn.ui

import android.view.View
import android.view.View.OnLayoutChangeListener
import android.view.ViewGroup.MarginLayoutParams
import kotlin.properties.Delegates.observable
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.LinearInterpolation
import net.mullvad.mullvadvpn.util.ListenableScrollableView

// In order to use this view controller, the parent view must contain four views with specific IDs:
//
// 1. A scroll area `View` with the `scrollAreaId` that implements `ListenableScrollableView`, which
//    is used to animate the title based on the scroll offset.
// 2. A view inside the scroll area with the ID `expanded_title`. This view is made invisible so
//    that it's not drawn, but it is used to measure the layout and the animation positions.
// 3. A view outside the scroll area with the ID `collapsed_title`. This view is also made
//    invisible just like the `expanded_view`.
// 4. A view with the ID `title`. This is the view that's actually drawn, and it's position and size
//    are interpolated from the expanded title to the collapsed title. This view should be placed
//    somewhere where it is drawn over all other views.
//
// The animation interpolation is calculated based on the Y scroll offset of the scroll area. Once
// the offset reaches a value that completely hides the expanded title inside the scroll view, the
// animation finishes with the title being in the collapsed state.
class CollapsibleTitleController(val parentView: View, scrollAreaId: Int = R.id.scroll_area) {
    private inner class LayoutListener(val listener: (View) -> Unit) : OnLayoutChangeListener {
        override fun onLayoutChange(
            view: View,
            left: Int,
            top: Int,
            right: Int,
            bottom: Int,
            oldLeft: Int,
            oldTop: Int,
            oldRight: Int,
            oldBottom: Int
        ) {
            listener.invoke(view)
            update()
        }
    }

    private val scaleInterpolation = LinearInterpolation()
    private val scrollInterpolation = LinearInterpolation()
    private val xOffsetInterpolation = LinearInterpolation()
    private val yOffsetInterpolation = LinearInterpolation()

    private val collapsedTitleLayoutListener: LayoutListener =
        LayoutListener() { collapsedTitle ->
            val (x, y) = calculateViewCoordinates(collapsedTitle)

            collapsedTitleHeight = collapsedTitle.height.toFloat()

            scaleInterpolation.end = collapsedTitleHeight / maxOf(1.0f, titleHeight)
            xOffsetInterpolation.end = x
            yOffsetInterpolation.end = y
        }

    private val collapsedTitleView =
        parentView.findViewById<View>(R.id.collapsed_title).apply {
            addOnLayoutChangeListener(collapsedTitleLayoutListener)
            visibility = View.INVISIBLE
        }

    private val expandedTitleLayoutListener: LayoutListener =
        LayoutListener() { expandedTitle ->
            val (x, y) = calculateViewCoordinates(expandedTitle)

            val expandedTitleMarginTop =
                when (val layoutParams = expandedTitle.layoutParams) {
                    is MarginLayoutParams -> layoutParams.topMargin
                    else -> 0
                }

            expandedTitleHeight = expandedTitle.height.toFloat()

            scaleInterpolation.start = expandedTitleHeight / maxOf(1.0f, titleHeight)
            xOffsetInterpolation.start = x
            yOffsetInterpolation.start = y

            scrollInterpolation.end = expandedTitleHeight + expandedTitleMarginTop
        }

    private val titleLayoutListener: LayoutListener =
        LayoutListener() { title ->
            val (x, y) = calculateViewCoordinates(title)

            titleWidth = title.width.toFloat()
            titleHeight = title.height.toFloat()

            scaleInterpolation.start = expandedTitleHeight / maxOf(1.0f, titleHeight)
            scaleInterpolation.end = collapsedTitleHeight / maxOf(1.0f, titleHeight)
            xOffsetInterpolation.reference = x
            yOffsetInterpolation.reference = y
        }

    private val titleView =
        parentView.findViewById<View>(R.id.title).apply {
            addOnLayoutChangeListener(titleLayoutListener)

            // Setting the scale pivot point to the left corner simplifies the calculations
            pivotX = 0.0f
            pivotY = 0.0f
        }

    private val scrollAreaLayoutListener: LayoutListener =
        LayoutListener() { scrollOffset = scrollArea.verticalScrollOffset.toFloat() }

    private val scrollArea =
        parentView.findViewById<View>(scrollAreaId).let { view ->
            val scrollableView = view as ListenableScrollableView

            view.addOnLayoutChangeListener(scrollAreaLayoutListener)

            scrollableView.onScrollListener = { _, top, _, _ ->
                scrollOffset = top.toFloat()
                update()
            }

            scrollableView
        }

    private var scrollOffsetUpdated = false
        get() {
            if (field == true) {
                field = false
                return true
            } else {
                return false
            }
        }

    private var collapsedTitleHeight = 0.0f
    private var expandedTitleHeight = 0.0f
    private var titleWidth = 0.0f
    private var titleHeight = 0.0f

    private var scrollOffset: Float by
        observable(0.0f) { _, old, new ->
            if (scrollOffsetUpdated == false && old != new) {
                scrollOffsetUpdated = true
            }
        }

    val fullCollapseScrollOffset: Float
        get() = scrollInterpolation.end

    var expandedTitleView by
        observable<View?>(null) { _, oldView, newView ->
            oldView?.removeOnLayoutChangeListener(expandedTitleLayoutListener)
            newView?.apply {
                addOnLayoutChangeListener(expandedTitleLayoutListener)
                expandedTitleLayoutListener.listener(this)
                visibility = View.INVISIBLE
            }
        }

    init {
        expandedTitleView = parentView.findViewById<View>(R.id.expanded_title)
        update()
    }

    fun onDestroy() {
        scrollArea.onScrollListener = null
        (scrollArea as View).removeOnLayoutChangeListener(scrollAreaLayoutListener)

        collapsedTitleView.removeOnLayoutChangeListener(collapsedTitleLayoutListener)
        expandedTitleView?.removeOnLayoutChangeListener(expandedTitleLayoutListener)
        titleView.removeOnLayoutChangeListener(titleLayoutListener)
    }

    private fun update() {
        val shouldUpdate =
            scrollOffsetUpdated ||
                scaleInterpolation.updated ||
                xOffsetInterpolation.updated ||
                yOffsetInterpolation.updated

        if (shouldUpdate) {
            val progress =
                if (expandedTitleView != null) {
                    maxOf(0.0f, minOf(1.0f, scrollInterpolation.progress(scrollOffset)))
                } else {
                    1.0f
                }

            val scale = scaleInterpolation.interpolate(progress)
            val offsetX = xOffsetInterpolation.interpolate(progress)
            val offsetY = yOffsetInterpolation.interpolate(progress)

            titleView.apply {
                scaleX = scale
                scaleY = scale
                translationX = offsetX
                translationY = offsetY
            }
        }
    }

    private fun calculateViewCoordinates(view: View): Pair<Float, Float> {
        var currentView = view
        var x = 0.0f
        var y = 0.0f

        while (currentView != parentView) {
            val parent = currentView.parent

            x += currentView.x - currentView.translationX
            y += currentView.y - currentView.translationY

            if (parent is View) {
                currentView = parent
            } else {
                break
            }
        }

        return Pair(x, y)
    }
}
