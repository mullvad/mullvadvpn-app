//
//  NotificationPromptView.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-01-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct NotificationPromptView<ViewModel>: View where ViewModel: NotificationPromptViewModelProtocol {

    @ObservedObject var viewModel: ViewModel
    @ScaledMetric private var iconSize: CGFloat = 48.0
    @State private var sizeOfView: CGSize = .zero

    var didConclude: (@MainActor (Bool) -> Void)? = nil

    init(viewModel: ViewModel, didConclude: @escaping @MainActor (Bool) -> Void) {
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

                                Text(text)
                                    .font(.mullvadLarge)
                                    .foregroundStyle(.white)
                                    .multilineTextAlignment(.center)
                            }

                        case .message(let message):
                            Text(message)
                                .font(.mullvadSmall)
                                .multilineTextAlignment(.center)
                                .foregroundStyle(.white.opacity(0.6))

                        case .emptyView:
                            Spacer()

                        case .action(let text, let style, let action):
                            MainButton(text: text, style: style, action: action)
                        }
                    }
                }
                .padding(.horizontal, 16)
                .padding(.bottom, 24)
                .frame(maxWidth: .infinity)
                .frame(minHeight: geo.size.height)
            }
            .background(Color.mullvadBackground)
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
