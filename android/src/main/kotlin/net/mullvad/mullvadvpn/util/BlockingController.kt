package net.mullvad.mullvadvpn

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

class BlockingController(val onSetEnable: (Boolean) -> Unit, val onClick: () -> Job) {
    var job: Job? = null
    var innerJob: Job? = null

    fun action() {
        if (!(job?.isActive ?: false)) {
            job = GlobalScope.launch(Dispatchers.Main) {
                onSetEnable(false)
                innerJob = onClick.invoke()
                innerJob?.join()
                onSetEnable(true)
            }
        }
    }

    fun onPause() {
        innerJob?.cancel()
        job?.cancel()
        onSetEnable(true)
    }

    fun onDestroy() {
        onPause()
    }
}
