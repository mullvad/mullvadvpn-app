import MullvadSettings
import SwiftUI

struct MultihopWhenNeededInfoView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var multihopWarningAlert: MullvadAlert?

    var body: some View {
        VStack(spacing: 16) {
            Spacer()

            Image.mullvadIconMultihopWhenNeeded
                .resizable()
                .frame(width: 48, height: 48)

            Group {
                Text(
                    "The entry server is currently selected automatically to ensure your current settings "
                        + "work with your selected location."
                )
                Text(
                    "To manually select an entry server, please switch multihop mode to “\("Always")”."
                )
            }
            .multilineTextAlignment(.center)
            .foregroundStyle(Color.mullvadTextSecondary)
            .font(.mullvadSmall)

            Spacer()

            MainButton(text: "Set multihop to “\("Always")“", style: .default) {
                if viewModel.filtersWillBeOverridden(.always) {
                    multihopWarningAlert = getMultihopFilterOverrideWarningAlert()
                } else if viewModel.multihopStateIsIncompatible(.always) {
                    multihopWarningAlert = getMultihopBlockedStateWarningAlert()
                } else {
                    viewModel.multihopState = .always
                }
            }
        }
        .padding()
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
                    type: .default,
                    title: "Continue",
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = .always
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .default,
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
                    type: .danger,
                    title: LocalizedStringKey(BlockedStateString.Button.multihop(.always).description),
                    identifier: AccessibilityIdentifier.multihopConfirmAlertEnableButton,
                    handler: {
                        viewModel.multihopState = .always
                        multihopWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .default,
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
