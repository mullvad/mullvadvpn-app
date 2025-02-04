//
//  GroupedRowView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-11-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct GroupedRowView<Content: View>: View {
    let content: Content

    init(@ViewBuilder _ content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        VStack(spacing: 0) {
            content
        }
        .background(Color(UIColor.primaryColor))
        .cornerRadius(UIMetrics.SettingsRowView.cornerRadius)
    }
}

#Preview("GroupedRowView") {
    StatefulPreviewWrapper((enabled: true, directOnly: false)) { values in
        GroupedRowView {
            SwitchRowView(isOn: values.enabled, text: "Enable")
            SwitchRowView(isOn: values.directOnly, text: "Direct only")
        }
    }
}
