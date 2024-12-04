//
//  SettingsRowViewFooter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SettingsRowViewFooter: View {
    let text: String

    var body: some View {
        Text(verbatim: text)
            .font(.footnote)
            .opacity(0.6)
            .foregroundStyle(Color(.primaryTextColor))
            .padding(UIMetrics.SettingsRowView.footerLayoutMargins)
    }
}
