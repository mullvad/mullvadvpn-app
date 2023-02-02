package net.mullvad.mullvadvpn.compose.screen

import android.app.Activity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.fragment.app.FragmentActivity
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.CustomDnsComposeCell
import net.mullvad.mullvadvpn.compose.component.MtuComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.theme.CollapsingToolbarTheme
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.ui.fragments.SplitTunnelingFragment
import net.mullvad.mullvadvpn.viewmodel.AdvancedSettingViewModel

@Composable
fun AdvancedSettingScreen(
    viewModel: AdvancedSettingViewModel,
    onBackClick: () -> Unit,
) {

    val uiState = viewModel.uiState.collectAsState().value

//    if (uiState.dnsSettings is DnsSettingsState.AddLocalDnsConfirm) {
//        ShowConfirmLocalDnsScreen(
//            viewModel = viewModel,
//            localDns = uiState.dnsSettings.newDns()
//        )
//    }

    CollapsingToolbarTheme {
        Surface(color = MaterialTheme.colors.background) {
            val state = rememberCollapsingToolbarScaffoldState()
            val progress = state.toolbarState.progress
            var activityContext = LocalContext.current as Activity

            val nestedScrollConnection = remember {
                object : NestedScrollConnection {
                    override suspend fun onPostFling(
                        consumed: Velocity,
                        available: Velocity
                    ): Velocity {
                        return super.onPostFling(consumed, available)
                    }

                    override suspend fun onPreFling(available: Velocity): Velocity {
                        return super.onPreFling(available)
                    }
                }
            }

            var enabled by remember { mutableStateOf(true) }

            var verticalSpacing = dimensionResource(id = R.dimen.vertical_space)
            var cellSideSpacing = dimensionResource(id = R.dimen.cell_left_padding)

            Box {
                CollapsingToolbarScaffold(
                    modifier = Modifier
                        .fillMaxSize(),
                    state = state,
                    scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
                    toolbarModifier = Modifier.background(MaterialTheme.colors.primary),
                    enabled = enabled,
                    toolbar = {

                        var scaffoldModifier = Modifier
                            .road(
                                whenCollapsed = Alignment.TopCenter,
                                whenExpanded = Alignment.BottomStart
                            )

                        CollapsingTopBar(
                            backgroundColor = MullvadDarkBlue,
                            onBackClicked = {
//                                customDnsAdapter?.stopEditing()
                                activityContext?.onBackPressed()
                            },
                            title = stringResource(id = R.string.settings_advanced),
                            progress = progress,
                            scaffoldModifier = scaffoldModifier,
                            backTitle = stringResource(id = R.string.settings),
                        )
                    }
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .background(MullvadDarkBlue)
                            .verticalScroll(state = rememberScrollState())
                            .nestedScroll(nestedScrollConnection)
                    ) {
                        ConstraintLayout(
                            modifier = Modifier
                                .fillMaxWidth()
                                .wrapContentHeight()
                                .padding(top = 8.dp)
                                .background(MullvadBlue)
                        ) {
//                            MtuComposeCell {}

                            MtuComposeCell(uiState.mtu, { newMtuValue ->
                                viewModel.onMtuChanged(newMtuValue)
                            }, { viewModel.onSubmitMtu() })
                        }
                        ConstraintLayout(
                            modifier = Modifier
                                .fillMaxWidth()
                                .wrapContentHeight()
                                .background(MullvadBlue)
                        ) {
                            NavigationComposeCell(
                                title = stringResource(id = R.string.split_tunneling),
                                onClick = {
                                    val fragment =
                                        SplitTunnelingFragment::class.java.getConstructor()
                                            .newInstance()

                                    (activityContext as? FragmentActivity)?.supportFragmentManager
                                        ?.beginTransaction()
                                        ?.apply {
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
                            )
                        }
                        ConstraintLayout(
                            modifier = Modifier
                                .fillMaxWidth()
                                .wrapContentHeight()
                                .padding(top = verticalSpacing)
                                .background(MullvadBlue)
                        ) {
                            var list = ArrayList<String>()
                            list.add("1.1.1.1")
                            list.add("2.2.2.2")
                            list.add("3.3.3.3")
                            list.add("0.0.0.0")
                            list.add("1.2.3.0")
                            CustomDnsComposeCell(
                                checkboxDefaultState = uiState.isCustomDnsEnabled,
                                onToggle = { viewModel.toggleCustomDns(it) },
                                dnsList = list,
                            )
                        }
                    }
                }
            }
        }
    }
}
