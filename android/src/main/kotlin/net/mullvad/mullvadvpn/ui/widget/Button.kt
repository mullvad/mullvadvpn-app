package net.mullvad.mullvadvpn.ui.widget

import android.content.Context
import android.util.AttributeSet
import net.mullvad.mullvadvpn.util.JobTracker

class Button : android.widget.Button {
    private var clickJobName: String? = null
    private var jobTracker: JobTracker? = null
    private var onClickAction: (suspend () -> Unit)? = null

    constructor(context: Context) : super(context) {}

    constructor(context: Context, attributes: AttributeSet) : super(context, attributes) {}

    constructor(context: Context, attributes: AttributeSet, defaultStyleAttribute: Int) :
        super(context, attributes, defaultStyleAttribute) {}

    constructor(
        context: Context,
        attributes: AttributeSet,
        defaultStyleAttribute: Int,
        defaultStyleResource: Int
    ) : super(context, attributes, defaultStyleAttribute, defaultStyleResource) {
    }

    override fun setEnabled(enabled: Boolean) {
        super.setEnabled(enabled)

        if (enabled) {
            alpha = 1.0f
        } else {
            alpha = 0.5f
        }
    }

    init {
        setOnClickListener {
            jobTracker?.newUiJob(clickJobName!!, onClickAction!!)
        }
    }

    fun setOnClickAction(jobName: String, tracker: JobTracker, action: suspend () -> Unit) {
        clickJobName = jobName
        jobTracker = tracker
        onClickAction = action
    }
}
