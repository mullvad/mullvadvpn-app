//
//  RevokedDeviceView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RevokedDeviceView: View {
    @StateObject var viewModel: RevokedDeviceViewModel
    var onLogout: (() -> Void)?

    var body: some View {
        VStack(spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    HStack {
                        Spacer()
                        Image.mullvadIconFail
                        Spacer()
                    }

                    Text("Device is inactive")
                        .font(.mullvadLarge)
                        .foregroundStyle(Color.MullvadText.onBackgroundEmphasis100)
                        .padding(.top, 16)

                    Text("You have removed this device. To connect again, you will need to log back in.")
                        .font(.mullvadSmall)
                        .foregroundStyle(Color.MullvadText.onBackground)
                        .padding(.top, 8)

                    Text("Going to login will unblock the Internet on this device.")
                        .font(.mullvadSmall)
                        .foregroundStyle(Color.MullvadText.onBackground)
                        .padding(.top, 16)
                        .showIf(viewModel.tunnelState.isSecured)
                }
                .padding(.top, 24)
            }
            .apply {
                $0.scrollBounceBehavior(.automatic)
            }

            MainButton(text: "Go to login", style: viewModel.tunnelState.isSecured ? .danger : .default) {
                onLogout?()
            }
            .accessibilityIdentifier(.revokedDeviceLoginButton)
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .background(Color.mullvadBackground)
    }
}

#Preview("Secured") {
    RevokedDeviceView(
        viewModel: RevokedDeviceViewModel(
            interactor: MockRevokedDeviceInteractor(
                tunnelStatus: TunnelStatus(
                    observedState: .error(.init(reason: .deviceRevoked)),
                    state: .error(.deviceRevoked)
                )
            )
        )
    )
}

#Preview("Not secured") {
    RevokedDeviceView(
        viewModel: RevokedDeviceViewModel(
            interactor: MockRevokedDeviceInteractor(
                tunnelStatus: TunnelStatus(
                    observedState: .disconnected,
                    state: .disconnected
                )
            )
        )
    )
}
