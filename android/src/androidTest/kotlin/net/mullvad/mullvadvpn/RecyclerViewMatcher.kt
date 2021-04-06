package net.mullvad.mullvadvpn

import android.content.res.Resources
import android.content.res.Resources.NotFoundException
import android.view.View
import androidx.recyclerview.widget.RecyclerView
import org.hamcrest.Description
import org.hamcrest.Matcher
import org.hamcrest.TypeSafeMatcher

class RecyclerViewMatcher(private val recyclerViewId: Int) {
    fun atPosition(position: Int): Matcher<View> {
        return atPositionOnView(position, -1)
    }

    fun atPositionOnView(position: Int, targetViewId: Int): Matcher<View> =
        object : TypeSafeMatcher<View>() {
            var resources: Resources? = null
            var childView: View? = null

            override fun describeTo(description: Description) {
                val idDescription = resources?.let {
                    try {
                        it.getResourceName(recyclerViewId)
                    } catch (var4: NotFoundException) {
                        "$recyclerViewId (resource name not found)"
                    }
                } ?: recyclerViewId.toString()
                description.appendText("with id: $idDescription")
            }

            override fun matchesSafely(view: View): Boolean {
                resources = view.resources
                if (childView == null) {
                    val recyclerView =
                        view.rootView.findViewById<View>(recyclerViewId) as RecyclerView?
                    childView = if (recyclerView != null && recyclerView.id == recyclerViewId) {
                        recyclerView.findViewHolderForAdapterPosition(position)?.itemView
                    } else {
                        return false
                    }
                }
                return if (targetViewId == -1) {
                    view == childView
                } else {
                    val targetView = childView?.findViewById<View>(targetViewId)
                    view == targetView
                }
            }
        }

    companion object {
        fun withRecyclerView(recyclerViewId: Int): RecyclerViewMatcher {
            return RecyclerViewMatcher(recyclerViewId)
        }
    }
}
