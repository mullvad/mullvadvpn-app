//
//  Untitled.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-04.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct MainButton: View {
    var text: String
    var style: MainButtonStyle.Style
    var disabled = false

    var action: () -> Void

    var body: some View {
        Button(action: action, label: {
            HStack {
                Spacer()
                Text(text)
                Spacer()
            }
        })
        .buttonStyle(MainButtonStyle(style, disabled: disabled))
        .cornerRadius(UIMetrics.MainButton.cornerRadius)
    }
}

#Preview {
    MainButton(text: "Connect", style: .success) {
        print("Tapped")
    }
}
