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

extension ProblemReportView {
    struct MessageEditOverlay: View {
        @State var viewModel: ProblemReportViewModelNew
        @FocusState var isFocused: Bool

        var body: some View {
            VStack(spacing: 0) {
                HStack {
                    Spacer()
                    Text("Report a problem")
                        .font(Font.system(.title2, design: .default))
                    Spacer()
                }
                .padding(16)
                .foregroundStyle(.white)
                .background(Color.mullvadBackground)
                ConfigurableTextView(
                    placeholder: ProblemReportView.messageTextViewPlaceholder,
                    text: $viewModel.message,
                    isFocused: $isFocused,
                    borderStyle: .constant(.none)
                )
                .padding(12)
                .background(Color.MullvadTextField.background)
                HStack {
                    Spacer()
                    Button("Done") { viewModel.isEditingMessage = false }
                }
                .padding(16)
                .background(Color.black)
            }
            .background(Color.mullvadBackground)
            .onAppear {
                isFocused = true
            }
        }
    }
}

#Preview {
    let viewModel = ProblemReportViewModelNew(interactor: MockProblemReportInteractor())
    return ProblemReportView.MessageEditOverlay(viewModel: viewModel)
}
