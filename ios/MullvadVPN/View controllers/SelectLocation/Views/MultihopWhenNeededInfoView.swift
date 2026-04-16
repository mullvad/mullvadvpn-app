import SwiftUI

struct MultihopWhenNeededInfoView<ViewModel: SelectLocationViewModel>: View {
    @ObservedObject var viewModel: ViewModel
    @State private var multihopBlockedStateWarningAlert: MullvadAlert?

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
                if viewModel.multihopStateIsIncompatible(.always) {
                    multihopBlockedStateWarningAlert = getMultihopBlockedStateWarningAlert()
                } else {
                    viewModel.multihopState = .always
                }
            }
        }
        .padding()
        .mullvadAlert(item: $multihopBlockedStateWarningAlert)
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
                        multihopBlockedStateWarningAlert = nil
                    }
                ),
                MullvadAlert.Action(
                    type: .default,
                    title: "Cancel",
                    handler: {
                        multihopBlockedStateWarningAlert = nil
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
