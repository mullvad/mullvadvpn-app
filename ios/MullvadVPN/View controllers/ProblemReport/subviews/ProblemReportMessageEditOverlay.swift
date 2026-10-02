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
        @State var viewModel: ProblemReportViewModel
        @FocusState var isFocused: Bool

        var body: some View {
            VStack(spacing: 0) {
                ZStack {
                    HStack {
                        Spacer()
                        Text("Problem description")
                            .font(.mullvadSmallSemiBold)
                        Spacer()
                    }
                    HStack {
                        Button {
                            viewModel.isEditingMessage = false
                        } label: {
                            ResizableImageView(image: .mullvadIconBack, dimension: .width(24))
                        }
                        Spacer()
                    }
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
            }
            .onAppear {
                isFocused = true
            }
        }
    }
}

#Preview {
    let viewModel = ProblemReportViewModel(interactor: MockProblemReportInteractor())
    return ProblemReportView.MessageEditOverlay(viewModel: viewModel)
}
