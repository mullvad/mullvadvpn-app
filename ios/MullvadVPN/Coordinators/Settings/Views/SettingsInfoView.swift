//
//  SettingsInfoView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-13.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SettingsInfoViewModel {
    let body: String
    let image: ImageResource
}

struct SettingsInfoView: View {
    let viewModel: SettingsInfoViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Image(viewModel.image)
                .resizable()
                .aspectRatio(contentMode: .fit)
            Text(viewModel.body)
                .font(.subheadline)
                .opacity(0.6)
        }
    }
}

#Preview {
    SettingsInfoView(viewModel: SettingsInfoViewModel(
        body: """
        Multihop routes your traffic into one WireGuard server and out another, making it \
        harder to trace. This results in increased latency but increases anonymity online.
        """,
        image: .multihopIllustration
    ))
}
