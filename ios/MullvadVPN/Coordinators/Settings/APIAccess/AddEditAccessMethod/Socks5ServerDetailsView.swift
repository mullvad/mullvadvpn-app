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

struct Socks5ServerDetailsView: View {
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

    private var messages: [String] {
        guard case .invalid(let validationError) = viewModel.validationState else {
            return []
        }

        return validationError.fieldErrors
            .filter {
                [.server, .port, .password, .cipher].contains($0.field)
            }
            .map(\.errorDescription)
    }

    var body: some View {
        Group {
            MullvadListSectionHeader(title: "Server details")
                .padding(.vertical, 12)

            serverField
            portField
            authenticateSwitch
            if authenticateBinding.wrappedValue {
                usernameField
                passwordField
            }
            errorMessageView
        }
        .onChange(of: viewModel.accessMethodViewModel.socks) {
            viewModel.clearErrors(for: [
                .server,
                .port,
                .username,
                .password,
            ])
        }
    }

    private var serverField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodServerCell,
            borderStyle: .constant(viewModel.borderStyle(for: .server)),
            leading: {
                AccessMethodLabelView(title: "Server", textColor: viewModel.borderStyle(for: .server).textColor)
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
            borderStyle: .constant(viewModel.borderStyle(for: .port)),
            leading: {
                AccessMethodLabelView(title: "Port", textColor: viewModel.borderStyle(for: .port).textColor)
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
                itemFactory.leading(for: .generic(title: NSLocalizedString("Authentication", comment: "")))
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
            borderStyle: .constant(viewModel.borderStyle(for: .username)),
            leading: {
                AccessMethodLabelView(title: "Username", textColor: viewModel.borderStyle(for: .username).textColor)
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
            borderStyle: .constant(viewModel.borderStyle(for: .password)),
            leading: {
                AccessMethodLabelView(title: "Password", textColor: viewModel.borderStyle(for: .password).textColor)
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

    private var errorMessageView: some View {
        ValidationErrorView(error: messages.joined(separator: "\n"))
    }
}
