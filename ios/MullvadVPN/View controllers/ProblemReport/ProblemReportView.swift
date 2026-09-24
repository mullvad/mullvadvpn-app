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

    let messageTextViewPlaceholder: LocalizedStringKey = """
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
        }.popover(isPresented: viewModel.showLogs) {
            ProblemReportView.LogView(viewModel: viewModel)
        }.mullvadAlert(item: $viewModel.alert)
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
            ConfigurableTextView(
                title: "Problem description",
                placeholder: messageTextViewPlaceholder,
                text: $viewModel.message,
                borderStyle: .constant(.normal)
            )
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
        .scrollable(fill: .vertical)
        .background(Color.mullvadBackground)
    }

}

// MARK: previews
struct MockInteractor: ProblemReportInteractorProtocol {
    var reportError: (any Error)?

    func fetchReportString(completion: @escaping @Sendable (String) -> Void) {
        completion(
            """
            The log file will go here
            =========================

            Something something something...
            """
        )
    }

    func sendReport(
        email: String, message: String, includeAccountTokenInLogs: Bool,
        completion: @escaping (Result<Void, any Error>) -> Void
    ) {
        //  try await Task.sleep(nanoseconds: 1_000_000_000)
        if let reportError {
            completion(.failure(reportError))
        } else {
            completion(.success(()))
        }
    }
}

#Preview {
    let viewModel = ProblemReportViewModelNew(interactor: MockInteractor())
    return ProblemReportView(viewModel: viewModel)
}
