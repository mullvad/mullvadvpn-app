// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct ProblemReportView: View {
    // temporary variables, until the view model is in place
    @State private var text = ""
    @State private var message: MessageView.Message?
    @State private var borderStyle: BorderStyle = .normal

    @State var viewModel: ProblemReportViewModelNew
    @State var showLogs: Bool = false

    @Namespace private var transitionNamespace
    private let messageEditTransitionId = "editFrame"

    let subheadLabelText: LocalizedStringKey = """
        To help you more effectively, your app’s log file will be attached \
        to this message. Your data will remain secure and private, as it \
        is anonymised before being sent over an encrypted channel.
        """

    let accountTokenInclusionPrompt: LocalizedStringKey =
        """
        Include my account token for faster help with \
        payment or account related issues
        """

    let emailPlaceholderText: LocalizedStringKey = "Your email (optional)"

    let messageTextViewTitle: LocalizedStringKey = "Problem description"
    static let messageTextViewPlaceholder: LocalizedStringKey = """
        To assist you better, please write in English or Swedish \
        and include which country you are connecting from.
        """

    var body: some View {
        Group {
            if let modalState = viewModel.modalState {
                ModalOverlay(state: modalState)
                    .accessibilityIdentifier(.problemReportSubmittedView)
            } else {
                mainForm
            }
        }
        .mullvadAlert(item: $viewModel.alert)
        .fullScreenCover(isPresented: $viewModel.isEditingMessage) {
            MessageEditOverlay(
                viewModel: viewModel
            )
            .apply {
                if #available(iOS 18.0, *) {
                    $0.navigationTransition(.zoom(sourceID: messageEditTransitionId, in: transitionNamespace))
                } else {
                    $0
                }
            }
        }
        .popover(isPresented: viewModel.showLogs) {
            LogView(viewModel: viewModel)
        }
    }

    var mainForm: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Report a problem")
                .font(.mullvadLarge)
                .foregroundStyle(.white)
            Text(subheadLabelText)
                .font(.mullvadTiny)
                .foregroundStyle(.white)
            ConfigurableTextField(
                title: "Email (optional)",
                placeholder: "Enter your email",
                text: $viewModel.email,
                borderStyle: .constant(.normal))
            messageEditInlinePlaceholder
            ActionBox(
                isChecked: $viewModel.includeAccountTokenInLogs,
                toggleTitle: accountTokenInclusionPrompt,
                additionalInfo: .init(
                    warningTitle: "This impacts your anonymity",
                    warningMessage:
                        "By attaching your account token it links this report to your account, which helps us resolve your issue quicker. All reports are automatically deleted after a period of time. For details, please see our **privacy policy**"
                )
            )
            MullvadButton(
                text: "View app logs",
                style: .primary,
                mainAccessibilityIdentifier: .problemReportAppLogsButton
            ) {
                viewModel.doShowLog()
            }
            MullvadButton(
                text: "Send",
                style: .success,
                mainAccessibilityIdentifier: .problemReportSendButton
            ) {
                viewModel.submitForm()
            }
            .disabled(!viewModel.canSend)
        }
        .padding(UIMetrics.padding16)
        .scrollable(fill: .vertical, alignment: .top)
        .background(Color.mullvadBackground)
    }

    // a mock text editing field which expands and morphs into a real text editor
    var messageEditInlinePlaceholder: some View {
        VStack(alignment: .leading, spacing: 0.0) {
            Text(messageTextViewTitle)
                .foregroundStyle(Color.MullvadTextField.textInput)
                .font(.mullvadTinySemiBold)
                .padding(.bottom, 4.0)
            Button {
                viewModel.isEditingMessage = true
            } label: {
                ZStack(alignment: .topLeading) {
                    HStack {
                        Text(ProblemReportView.messageTextViewPlaceholder)
                            .foregroundStyle(Color.MullvadTextField.inputPlaceholder)
                            .opacity(viewModel.message.isEmpty ? 1.0 : 0.0)
                        Spacer()
                    }
                    Text(viewModel.message)
                        .foregroundStyle(Color.MullvadTextField.textInput)
                }
                .multilineTextAlignment(.leading)
                .padding(.horizontal, 8.0)
                .padding(.vertical, 12.0)
                .background { Color.MullvadTextField.background }
                .font(.mullvadSmall)
                .apply {
                    if #available(iOS 18.0, *) {
                        $0.matchedTransitionSource(id: messageEditTransitionId, in: transitionNamespace) {
                            $0.background(Color.MullvadTextField.background)
                        }
                    } else {
                        $0
                    }
                }
                .modifier(
                    RoundedCornerModifier(
                        cornerRadius: 4.0,
                        corners: .allCorners,
                        insertBy: .zero,
                        borderColor: Color.MullvadTextField.border,
                        borderWidth: 1.0
                    )
                )
            }
        }
    }
}

#Preview {
    ProblemReportView(
        viewModel: ProblemReportViewModelNew(
            interactor: MockProblemReportInteractor()
        )
    )
}
