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

struct Socks5ServerDeitalsView: View {
    let itemFactory = SegmentedListItemFactory()
    let viewModel: any ConfigureApiAccessMethodViewModelProtocol

    private var serverBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.socks.server },
            set: { viewModel.accessMethodViewModel.socks.server = $0 }
        )
    }

    private var portBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.socks.port },
            set: { viewModel.accessMethodViewModel.socks.port = $0 }
        )
    }

    private var authenticateBinding: Binding<Bool> {
        Binding(
            get: { viewModel.accessMethodViewModel.socks.authenticate },
            set: { viewModel.accessMethodViewModel.socks.authenticate = $0 }
        )
    }

    var body: some View {
        MullvadListSectionHeader(title: "Server details")
            .padding(.vertical, 12.0)
        serverField
        portField
        authenticateSwitch
        if authenticateBinding.wrappedValue {
            usernameField
            passwordField
        }
    }

    private var serverField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodServerCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Server"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: serverBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear, showsClearButton: false)
                )
            }
        )
        .padding(.bottom, 1.0)
        .environment(\.isNestedInSegmentedListItem, false)
    }

    private var portField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Port"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: portBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear, showsClearButton: false),
                    configuration: TextFieldNamespace.Configuration(
                        formatter: GroupedTextFormatter.portNumber, keyboardType: .numberPad)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
        .padding(.bottom, 1.0)
    }

    var authenticateSwitch: some View {
        SegmentedListItem(
            isLastInList: authenticateBinding.wrappedValue == false,
            accessibilityIdentifier: .socks5AuthenticationSwitch,
            leading: {
                itemFactory.leading(for: .generic(title: "Authentication"))
            },
            trailing: {
                itemFactory.trailing(for: .toggle(isOn: authenticateBinding, isDisabled: false))
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
        .padding(.bottom, 1.0)
    }

    private var usernameField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Username"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: portBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear, showsClearButton: false),
                    configuration: TextFieldNamespace.Configuration(
                        formatter: GroupedTextFormatter.portNumber, keyboardType: .numberPad)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
        .padding(.bottom, 1.0)
    }

    private var passwordField: some View {
        SegmentedListItem(
            isLastInList: true,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: "Password"))
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Required"),
                    text: portBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear, showsClearButton: false),
                    configuration: TextFieldNamespace.Configuration(
                        formatter: GroupedTextFormatter.portNumber, keyboardType: .numberPad)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
    }
}
