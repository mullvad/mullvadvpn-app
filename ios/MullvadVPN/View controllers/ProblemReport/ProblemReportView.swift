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

    let subheadLabelText: LocalizedStringKey = "To help you more effectively, your app’s log file will be attached to this message. Your data will remain secure and private, as it is anonymised before being sent over an encrypted channel."
    
    let accountTokenInclusionPrompt: LocalizedStringKey =
        "Include my account token for faster help with payment or account related issues"
    
    let emailPlaceholderText: LocalizedStringKey = "Your email (optional)"

    let messageTextViewPlaceholder: LocalizedStringKey = "To assist you better, please write in English or Swedish and include which country you are connecting from."
    
    let emptyEmailAlertWarning : LocalizedStringKey = "You are about to send the problem report without a way for us to get back to you. If you want an answer to your report you will have to enter an email address."
    
    var body: some View {
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
                        "By attaching your account token it links this report to your account, which helps us resolve your issue quicker. All reports are automatically deleted after a period of time. For details, please see our _privacy policy_"
                )
            )
            MullvadButton(text: "View app logs", style: .primary) {
            }
            MullvadButton(text: "Send", style: .success) {
            }
                .disabled(!viewModel.canSend)
        }
        .padding(UIMetrics.padding16)
        .scrollable(fill: .vertical)
        .background(Color.mullvadBackground)
    }
}

#Preview {
    let viewModel = ProblemReportViewModelNew()
    ProblemReportView(viewModel: viewModel)
}
