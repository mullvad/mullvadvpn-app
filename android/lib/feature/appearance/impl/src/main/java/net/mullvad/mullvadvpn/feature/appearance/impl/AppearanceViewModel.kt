package net.mullvad.mullvadvpn.feature.appearance.impl

import android.app.StatusBarManager.TILE_ADD_REQUEST_ERROR_NOT_CURRENT_USER
import android.app.StatusBarManager.TILE_ADD_REQUEST_ERROR_NO_STATUS_BAR_SERVICE
import android.app.StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ADDED
import android.app.StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ALREADY_ADDED
import android.app.StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_NOT_ADDED
import androidx.lifecycle.ViewModel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow
import net.mullvad.mullvadvpn.feature.appearance.impl.AppearanceSideEffect.RequestAddTileResult

class AppearanceViewModel : ViewModel() {

    private val _uiSideEffect = Channel<AppearanceSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    fun onRequestAddTileResult(result: Int) {
        result.toTileSideEffect()?.let { _uiSideEffect.trySend(it) }
    }

    private fun Int.toTileSideEffect(): RequestAddTileResult? =
        when (this) {
            TILE_ADD_REQUEST_RESULT_TILE_ALREADY_ADDED -> RequestAddTileResult.TileAlreadyAdded
            TILE_ADD_REQUEST_RESULT_TILE_ADDED -> RequestAddTileResult.TileAdded
            TILE_ADD_REQUEST_ERROR_NOT_CURRENT_USER -> RequestAddTileResult.NotCurrentUser
            TILE_ADD_REQUEST_ERROR_NO_STATUS_BAR_SERVICE -> RequestAddTileResult.NoStatusBarService
            TILE_ADD_REQUEST_RESULT_TILE_NOT_ADDED -> null
            else -> RequestAddTileResult.Unknown(this)
        }
}

sealed interface AppearanceSideEffect {
    sealed interface RequestAddTileResult : AppearanceSideEffect {
        data object TileAdded : RequestAddTileResult

        data object TileAlreadyAdded : RequestAddTileResult

        data object NotCurrentUser : RequestAddTileResult

        data object NoStatusBarService : RequestAddTileResult

        data class Unknown(val result: Int) : RequestAddTileResult
    }
}
