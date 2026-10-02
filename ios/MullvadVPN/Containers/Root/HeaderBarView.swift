// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI
import UIKit

// Various fixes (eg. viewModel.onHeightChange, viewModel.safeAreaTop) can be removed
// once HeaderBarView isn't wrapped in a UIHostingViewController.
struct HeaderBarView: View {
    private let barButtonTappableAreaSize = UIMetrics.Button.minimumTappableAreaSize
    @State private var logoTrailingPoint: CGFloat = 0
    @State private var showBrandName: Bool = true

    let viewModel: HeaderBarViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Image.mullvadLogoImage
                    .resizable()
                    .scaledToFit()
                    .frame(height: UIMetrics.headerBarLogoSize)
                    .padding(.top, 4)
                Image.mullvadLogoText
                    .resizable()
                    .scaledToFit()
                    .tint(.mullvadTextPrimary)
                    .frame(height: UIMetrics.headerBarBrandNameHeight)
                    .background(determineBrandNameOverlap())
                    .showIf(showBrandName)

                Spacer()

                Button {
                    viewModel.onAccountTap?()
                } label: {
                    Image.mullvadIconAccount
                        .resizable()
                        .scaledToFit()
                        .frame(height: UIMetrics.Button.barButtonSize)
                }
                .frame(width: barButtonTappableAreaSize.width, height: barButtonTappableAreaSize.height)
                .accessibilityIdentifier(.accountButton)
                .accessibilityLabel(NSLocalizedString("Account", comment: ""))
                .showIf(viewModel.showAccountButton)

                Button {
                    viewModel.onSettingsTap?()
                } label: {
                    Group {
                        if let breadcrumb = viewModel.breadcrumb {
                            breadcrumb.settingsImage
                                .resizable()
                        } else {
                            Image.mullvadIconSettings
                                .resizable()
                        }
                    }
                    .scaledToFit()
                    .frame(height: UIMetrics.Button.barButtonSize)
                }
                .frame(width: barButtonTappableAreaSize.width, height: barButtonTappableAreaSize.height)
                .accessibilityIdentifier(.settingsButton)
                .accessibilityLabel(NSLocalizedString("Settings", comment: ""))
            }

            HStack(spacing: 16) {
                Text(viewModel.deviceName)
                    .accessibilityIdentifier(.headerDeviceNameLabel)
                Text(viewModel.timeLeft)
            }
            .showIf(viewModel.showDeviceInfo)
            .foregroundStyle(Color.mullvadTextSecondary)
            .font(.mullvadMiniSemiBold)
        }
        .padding(EdgeInsets(top: viewModel.safeAreaTop, leading: 16, bottom: 8, trailing: 10))
        .accessibilityElement(children: .contain)
        .background(Color(viewModel.backgroundColor))
        .ignoresSafeArea(edges: .top)
        .sizeOfView { viewModel.onHeightChange?($0.height) }
    }

    private func determineBrandNameOverlap() -> GeometryReader<some View> {
        GeometryReader { proxy in
            let frame = proxy.frame(in: .global)

            Color.clear
                .onAppear {
                    logoTrailingPoint = frame.origin.x + frame.width
                }
                .onChange(
                    of: frame,
                    { oldValue, newValue in
                        showBrandName = newValue.origin.x >= logoTrailingPoint
                    }
                )
        }
    }
}

#Preview {
    HeaderBarView(
        viewModel: HeaderBarViewModel(
            rootConfiguration: RootConfiguration(
                deviceName: "Happy Cow",
                expiry: Date().addingTimeInterval(60),
                showsAccountButton: true
            ),
            headerBarPresentation: HeaderBarPresentation(
                style: .default
            ),
            breadcrumb: Breadcrumb.error(.apiAccess)
        )
    )
}
