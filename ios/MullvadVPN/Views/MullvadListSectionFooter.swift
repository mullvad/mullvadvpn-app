//
//  MullvadListSectionFooter.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MullvadListSectionFooter: View {
    let title: LocalizedStringKey
    var body: some View {
        Text(title)
            .font(.mullvadMini)
            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.bottom, 24)
    }
}
#Preview {
    MullvadListSectionFooter(title: "Custom lists")
}
