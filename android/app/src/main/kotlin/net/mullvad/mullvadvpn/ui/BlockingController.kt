package net.mullvad.mullvadvpn.ui

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class BlockingController(val blockableView: BlockableView) {
    var job: Job? = null
    var innerJob: Job? = null

    fun action() {
        if (!(job?.isActive ?: false)) {
            job =
                GlobalScope.launch(Dispatchers.Main) {
                    blockableView.setEnabled(false)
                    innerJob = blockableView.onClick()
                    innerJob?.join()
                    blockableView.setEnabled(true)
                }
        }
    }

    fun onPause() {
        innerJob?.cancel()
        job?.cancel()
        blockableView.setEnabled(true)
    }

    fun onDestroy() {
        onPause()
    }
}

interface BlockableView {
    fun setEnabled(enabled: Boolean)
    fun onClick(): Job
}
