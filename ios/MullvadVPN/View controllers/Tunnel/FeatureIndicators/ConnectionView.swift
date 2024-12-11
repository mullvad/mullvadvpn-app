//
//  ConnectionView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-03.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

// TODO: Replace all hardcoded values with real values dependent on tunnel state. To be addressed in upcoming PR.

struct ConnectionView: View {
    var body: some View {
        ZStack {
            BlurView()

            VStack(alignment: .leading, spacing: 16) {
                ConnectionPanel()
                FeaturesIndicatoresView(viewModel: FeaturesIndicatoresMockViewModel())
                ButtonPanel()
            }
            .padding(16)
        }
        .cornerRadius(12)
        .padding(16)
        // Importing UIView in SwitftUI (see BlurView) has sizing limitations, so we need to help the view
        // understand its width constraints.
        .frame(maxWidth: UIScreen.main.bounds.width)
    }
}

#Preview {
    ConnectionView()
        .background(UIColor.secondaryColor.color)
}

private struct BlurView: View {
    var body: some View {
        Spacer()
            .overlay {
                VisualEffectView(effect: UIBlurEffect(style: .dark))
                    .opacity(0.8)
            }
    }
}

private struct ConnectionPanel: View {
    var body: some View {
        VStack(alignment: .leading) {
            Text("Connected")
                .textCase(.uppercase)
                .font(.title3.weight(.semibold))
                .foregroundStyle(UIColor.successColor.color)
                .padding(.bottom, 4)
            Text("Country, City")
                .font(.title3.weight(.semibold))
                .foregroundStyle(UIColor.primaryTextColor.color)
            Text("Server")
                .font(.body)
                .foregroundStyle(UIColor.primaryTextColor.color.opacity(0.6))
        }
    }
}

private struct ButtonPanel: View {
    var body: some View {
        VStack(spacing: 16) {
            SplitMainButton(
                text: "Switch location",
                image: .iconReload,
                style: .default,
                primaryAction: {
                    print("Switch location tapped")
                }, secondaryAction: {
                    print("Reload tapped")
                }
            )

            MainButton(
                text: "Cancel",
                style: .danger
            ) {
                print("Cancel tapped")
            }
        }
    }
}
