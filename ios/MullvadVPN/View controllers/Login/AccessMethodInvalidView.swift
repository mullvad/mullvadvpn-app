//
//  AccessMethodInvalidView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct AccessMethodInvalidView: View {
    var didPressButton: () -> Void

    var body: some View {
        ZStack {
            VStack(spacing: 16) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Custom API access method is invalid")
                        .font(.mullvadSmallSemiBold)
                    Text("Please update it or enable a different one to be able to reach the API using this method.")
                        .font(.mullvadTinySemiBold)
                }
                MainButton(text: "API access methods", style: .default) {
                    didPressButton()
                }
            }
            .padding(UIMetrics.Dashboard.padding)
        }
        .background(Color.MullvadDashboard.background)
        .foregroundStyle(Color.white)
        .cornerRadius(UIMetrics.Dashboard.cornerRadius)
    }
}

#Preview {
    AccessMethodInvalidView {
        print("Pressed button")
    }
}
