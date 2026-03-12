package net.mullvad.mullvadvpn.feature.addtime.impl.navigation

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.navigation3.runtime.EntryProviderScope
import net.mullvad.mullvadvpn.core.nav3.BottomSheetSceneStrategy
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.AddTimeNavKey
import net.mullvad.mullvadvpn.feature.addtime.impl.AddTimeBottomSheet

@OptIn(ExperimentalMaterial3Api::class)
fun EntryProviderScope<NavKey2>.addTimeEntry(navigator: Navigator) {
    entry<AddTimeNavKey>(
        metadata = BottomSheetSceneStrategy.bottomSheet()
    ) { navKey ->
        AddTimeBottomSheet(
            visible = navKey.visible,
            onRedeemVoucherClick = {},
            onPlayPaymentInfoClick = {},
            onHideBottomSheet = {},
        )
    }
}
