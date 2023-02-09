package net.mullvad.mullvadvpn.ui.fragment

import android.view.animation.Animation
import android.view.animation.AnimationUtils
import androidx.annotation.LayoutRes
import androidx.core.view.ViewCompat
import androidx.fragment.app.Fragment
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emptyFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.transitionFinished

abstract class BaseFragment : Fragment {
    constructor() : super()
    constructor (@LayoutRes contentLayoutId: Int) : super(contentLayoutId)

    protected var transitionFinishedFlow: Flow<Unit> = emptyFlow()
        private set

    override fun onCreateAnimation(transit: Int, enter: Boolean, nextAnim: Int): Animation? {
        val zAdjustment = if (animationsToAdjustZorder.contains(nextAnim)) {
            1f
        } else {
            0f
        }
        ViewCompat.setTranslationZ(requireView(), zAdjustment)
        return if (nextAnim != 0 && enter) {
            AnimationUtils.loadAnimation(context, nextAnim)?.apply {
                transitionFinishedFlow = transitionFinished()
            }
        } else {
            super.onCreateAnimation(transit, enter, nextAnim)
        }
    }

    companion object {
        private val animationsToAdjustZorder = listOf(
            R.anim.fragment_enter_from_right,
            R.anim.fragment_exit_to_right,
            R.anim.fragment_enter_from_bottom,
            R.anim.fragment_exit_to_bottom
        )
    }
}
