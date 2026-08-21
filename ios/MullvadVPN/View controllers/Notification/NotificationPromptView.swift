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

struct NotificationPromptView<ViewModel>: View where ViewModel: NotificationPromptViewModelProtocol {
    @ObservedObject var viewModel: ViewModel
    @ScaledMetric private var iconSize: CGFloat = 48.0
    @State private var sizeOfView: CGSize = .zero

    var didConclude: ((Bool) -> Void)? = nil

    init(viewModel: ViewModel, didConclude: @escaping (Bool) -> Void) {
        self.viewModel = viewModel
        self.didConclude = didConclude
    }

    var body: some View {
        GeometryReader { geo in
            ScrollView {
                VStack(spacing: 16) {
                    ForEach(viewModel.rows) { item in
                        switch item {
                        case .header(let image, let text):
                            VStack(spacing: 16) {
                                image
                                    .resizable()
                                    .frame(width: iconSize, height: iconSize)
                                    .foregroundStyle(Color(.primaryTextColor))

                                Text(text)
                                    .font(.mullvadLarge)
                                    .foregroundStyle(.white)
                                    .multilineTextAlignment(.center)
                            }

                        case .message(let message, let font):
                            Text(message)
                                .font(font)
                                .multilineTextAlignment(.center)
                                .foregroundStyle(.white.opacity(0.6))

                        case .emptyView:
                            Spacer()

                        case .action(let text, let style, let accessibilityIdentifier, let action):
                            MullvadButton(text: text, style: style, action: action)
                                .accessibilityIdentifier(accessibilityIdentifier)
                        }
                    }
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 24)
                .frame(maxWidth: .infinity)
                .frame(minHeight: geo.size.height)
            }
            .background(Color.mullvadBackground)
            .onAppear(perform: {
                viewModel.checkNotificationPermission()
            })
            .onChange(of: viewModel.isNotificationsAllowed) { oldValue, newValue in
                guard oldValue != newValue else { return }
                self.didConclude?(newValue)
            }
            .onChange(of: viewModel.isSkipped) { oldValue, newValue in
                guard oldValue != newValue else { return }
                self.didConclude?(false)
            }
            .onReceive(NotificationCenter.default.publisher(for: UIApplication.willEnterForegroundNotification)) { _ in
                viewModel.checkNotificationPermission()
            }
        }
    }
}

#Preview {
    NotificationPromptView(
        viewModel: NotificationPromptViewModel()
    ) { isGranted in
        print("Notification permission is \(isGranted ? "granted" : "denied")")
    }
}
