package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import android.widget.ImageView
import androidx.fragment.app.Fragment
import androidx.fragment.app.FragmentActivity
import kotlin.reflect.KClass
import net.mullvad.mullvadvpn.R

open class NavigateCell : Cell {
    private val chevron = ImageView(context).apply {
        val width = resources.getDimensionPixelSize(R.dimen.chevron_width)
        val height = resources.getDimensionPixelSize(R.dimen.chevron_height)

        layoutParams = LayoutParams(width, height, 0.0f)
        alpha = 0.6f

        setImageResource(R.drawable.icon_chevron)
    }

    var targetFragment: KClass<out Fragment>? = null

    constructor(context: Context) : super(context)

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes)

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute)

    init {
        cell.addView(chevron)
        onClickListener = { openSubFragment() }
    }

    private fun openSubFragment() {
        targetFragment?.let { fragmentClass ->
            val fragment = fragmentClass.java.getConstructor().newInstance()

            (context as? FragmentActivity)?.supportFragmentManager?.beginTransaction()?.apply {
                setCustomAnimations(
                    R.anim.fragment_enter_from_right,
                    R.anim.fragment_exit_to_left,
                    R.anim.fragment_half_enter_from_left,
                    R.anim.fragment_exit_to_right
                )
                replace(R.id.main_fragment, fragment)
                addToBackStack(null)
                commitAllowingStateLoss()
            }
        }
    }
}
