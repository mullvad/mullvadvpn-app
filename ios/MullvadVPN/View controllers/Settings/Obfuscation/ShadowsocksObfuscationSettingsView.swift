//
//  ShadowsocksObfuscationSettingsView.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2024-11-07.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct ShadowsocksObfuscationSettingsView<VM>: View where VM: ShadowsocksObfuscationSettingsViewModel {
    @StateObject var viewModel: VM

    @State var customValue = ""

    let title = "Shadowsocks port"

    func row<V: View>(isSelected: Bool, @ViewBuilder items: () -> V) -> some View {
        HStack {
            Image("IconTick").opacity(isSelected ? 1.0 : 0.0)
            items()
        }
        .padding(16)
        .background(isSelected ? Color(UIColor.Cell.Background.selected) : Color(UIColor.Cell.Background.normal))
        .foregroundColor(Color(UIColor.Cell.titleTextColor))
    }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text(title)
                Spacer()
            }
            row(isSelected: true) {
                Text("Automatic")
                Spacer()
            }
            row(isSelected: false) {
                Text("Custom")
                Spacer()
                TextField("value", text: $customValue, prompt: Text("Port        "))
                    .fixedSize().background(.white)
            }
            Spacer()
        }
        .background(Color(.secondaryColor))
        .foregroundColor(Color(.primaryTextColor))

        Spacer()
    }
}

#Preview {
    var model = MockShadowsocksObfuscationSettingsViewModel(shadowsocksPort: .automatic)
    return ShadowsocksObfuscationSettingsView(viewModel: model)
}
