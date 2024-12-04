//
//  SettingsInfoContainerView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SettingsInfoContainerView<Content: View>: View {
    let content: Content

    init(@ViewBuilder _ content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        ZStack {
            List {
                content
                    .listRowInsets(EdgeInsets())
                    .listRowSeparator(.hidden)
                    .listRowBackground(Color(.secondaryColor))
                    .padding(.top, UIMetrics.contentInsets.top)
                    .padding(.bottom, UIMetrics.contentInsets.bottom)
            }
            .listStyle(.plain)
        }
        .background(Color(.secondaryColor))
    }
}

#Preview {
    SettingsInfoContainerView {
        SettingsMultihopView(tunnelViewModel: MockMultihopTunnelSettingsViewModel())
    }
}
