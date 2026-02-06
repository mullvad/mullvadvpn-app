//
//  DAITAMultihopNotice.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-06-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct DAITAMultihopNotice: View {
    var body: some View {
        HStack(spacing: 8) {
            Image(.iconInfo)
                .resizable()
                .frame(width: 18, height: 18)
                .foregroundStyle(Color(.primaryTextColor).opacity(0.6))
            Text(NSLocalizedString("Multihop is being used to enable DAITA for your selected location.", comment: ""))
                .font(.mullvadTinySemiBold)
                .foregroundColor(Color(.primaryTextColor).opacity(0.6))
        }
    }
}

#Preview {
    SettingsInfoContainerView {
        DAITAMultihopNotice()
    }
}
