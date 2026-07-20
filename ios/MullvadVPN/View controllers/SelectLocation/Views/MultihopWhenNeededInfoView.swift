import MullvadSettings
import SwiftUI

struct MultihopWhenNeededInfoView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var multihopWarningAlert: MullvadAlert?

    var body: some View {
        MullvadStateView(
            viewModel: StateViewModel(
                style: .custom(.init(image: Image.mullvadIconMultihopWhenNeeded)),
                title: .init(
                    text: NSLocalizedString(
                        "The entry server is currently selected automatically to ensure your current settings work with your selected location.",
                        comment: ""),
                    style: .secondary(alignment: .center)),
                details: [
                    .init(
                        text: String(
                            format: NSLocalizedString(
                                "To manually select an entry server, please switch multihop mode to “%@”.",
                                comment: ""), MultihopStateV2.always.description), style: .secondary(alignment: .center)
                    )
                ],
                actions: [
                    MullvadStateView.ActionItem(
                        style: .primary,
                        state: .init(
                            kind: .idle,
                            message: String(
                                format: NSLocalizedString("Set multihop to “%@”", comment: ""),
                                MultihopStateV2.always.description)),
                        onTap: {
                            if viewModel.filtersWillBeOverridden(.always) {
                                multihopWarningAlert = getMultihopFilterOverrideWarningAlert()
                            } else if viewModel.multihopStateIsIncompatible(.always) {
                                multihopWarningAlert = getMultihopBlockedStateWarningAlert()
                            } else {
                                viewModel.multihopState = .always
                            }
                        })
                ])
        )
        .mullvadAlert(item: $multihopWarningAlert)
    }

    private func getMultihopFilterOverrideWarningAlert() -> MullvadAlert? {
        MullvadAlert(
            type: .warning,
            messages: [
                LocalizedStringKey(
                    String(
                        format: NSLocalizedString(
                            "You currently have entry filters applied. Switching to “%@“, the app will ignore filter "
                                + "settings for the entry server that is being automatically selected.",
                            comment: "Variable refers to multihop mode"
                        ),
                        MultihopState.always.description
                    )
                )
            ],
            actions: [
                MullvadAlert.Action(
                    type: .primary,
                    title: "Continue",
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = .always
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        multihopWarningAlert = nil
                    }
                ),
            ]
        )
    }

    private func getMultihopBlockedStateWarningAlert() -> MullvadAlert? {
        MullvadAlert(
            type: .warning,
            messages: [LocalizedStringKey(BlockedStateString.Message.multihop.description)],
            actions: [
                MullvadAlert.Action(
                    type: .destructivePrimary,
                    title: LocalizedStringKey(BlockedStateString.Button.multihop(.always).description),
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = .always
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        multihopWarningAlert = nil
                    }
                ),
            ]
        )
    }
}

#Preview {
    MultihopWhenNeededInfoView(viewModel: MockSelectLocationViewModel())
        .background(Color.mullvadBackground)
}
