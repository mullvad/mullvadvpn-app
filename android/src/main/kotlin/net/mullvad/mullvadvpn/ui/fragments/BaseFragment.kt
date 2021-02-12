package net.mullvad.mullvadvpn.ui.fragments

import android.view.animation.Animation
import androidx.core.view.ViewCompat
import androidx.fragment.app.Fragment
import net.mullvad.mullvadvpn.R

abstract class BaseFragment : Fragment() {
    override fun onCreateAnimation(transit: Int, enter: Boolean, nextAnim: Int): Animation? {
        val zAdjustment = if (animationsToAdjustZorder.contains(nextAnim)) {
            1f
        } else {
            0f
        }
        ViewCompat.setTranslationZ(requireView(), zAdjustment)
        return super.onCreateAnimation(transit, enter, nextAnim)
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
