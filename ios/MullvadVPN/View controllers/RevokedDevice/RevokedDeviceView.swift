//
//  RevokedDeviceView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RevokedDeviceView: View {
//    @ObservedObject var tunnelState: TunnelState
    var onButtonTap: (() -> Void)?

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
                }
                .padding(.top, 24)
            }
            .apply {
                if #available(iOS 16.4, *) {
                    $0.scrollBounceBehavior(.automatic)
                } else {
                    $0
                }
            }

            MainButton(text: "Go to login", style: .default) {
                onButtonTap?()
            }
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
        .background(Color.mullvadBackground)
    }
}

#Preview {
    RevokedDeviceView()
}
