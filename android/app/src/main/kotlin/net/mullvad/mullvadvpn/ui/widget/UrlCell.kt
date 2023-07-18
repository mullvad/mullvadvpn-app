package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.util.AttributeSet
import android.widget.ImageView
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.util.appendHideNavOnReleaseBuild

open class UrlCell : Cell {
    private val externalLinkIcon =
        ImageView(context).apply {
            layoutParams = LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT, 0.0f)
            alpha = 0.6f

            setImageResource(R.drawable.icon_extlink)
        }

    var url: Uri? = null

    @JvmOverloads
    constructor(
        context: Context,
        attributes: AttributeSet? = null,
        defaultStyleAttribute: Int = 0,
        defaultStyleResource: Int = 0
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
        loadAttributes(attributes)

        cell.addView(externalLinkIcon)

        onClickListener = { openLink() }
    }

    internal fun disableExternalLink() {
        externalLinkIcon.visibility = GONE
        onClickListener = null
    }

    private fun loadAttributes(attributes: AttributeSet?) {
        context.theme.obtainStyledAttributes(attributes, R.styleable.Url, 0, 0).apply {
            try {
                getString(R.styleable.Url_url)?.let { urlString ->
                    url = Uri.parse(urlString.appendHideNavOnReleaseBuild())
                }
            } finally {
                recycle()
            }
        }
    }

    private fun openLink() {
        url?.let { url ->
            val intent = Intent(Intent.ACTION_VIEW, url)

            context.startActivity(intent)
        }
    }
}
