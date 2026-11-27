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
    var focusedInput: FocusState<AccessMethodField?>.Binding

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

    private var usernameBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.socks.username },
            set: { viewModel.accessMethodViewModel.socks.username = $0 }
        )
    }

    private var passwordBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.socks.password },
            set: { viewModel.accessMethodViewModel.socks.password = $0 }
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
    }

    private var serverField: some View {
        SegmentedListItem(
            isLastInList: false,
            appearance: viewModel.hasError(for: .server) == nil ? .normal : .error,
            accessibilityIdentifier: .addAccessMethodServerCell,
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Server", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .input(
                        title: "",
                        placeholder: NSLocalizedString("Required", comment: ""),
                        text: serverBinding)
                )
                .focused(focusedInput, equals: .server)
            },
            onSelect: {
                focusedInput.wrappedValue = .server
            }
        )
        .padding(.bottom, 1.0)
        .environment(\.isNestedInSegmentedListItem, false)
    }

    private var portField: some View {
        SegmentedListItem(
            isLastInList: false,
            appearance: viewModel.hasError(for: .port) == nil ? .normal : .error,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Port", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .input(
                        title: "",
                        placeholder: NSLocalizedString("Required", comment: ""),
                        text: portBinding,
                        keyboardType: .numberPad)
                )
                .focused(focusedInput, equals: .port)
            },
            onSelect: {
                focusedInput.wrappedValue = .port
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
            appearance: viewModel.hasError(for: .username) == nil ? .normal : .error,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Username", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .input(
                        title: "",
                        placeholder: NSLocalizedString("Required", comment: ""),
                        text: usernameBinding)
                )
                .focused(focusedInput, equals: .username)
            },
            onSelect: {
                focusedInput.wrappedValue = .username
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
        .padding(.bottom, 1.0)
    }

    private var passwordField: some View {
        SegmentedListItem(
            isLastInList: true,
            appearance: viewModel.hasError(for: .password) == nil ? .normal : .error,
            accessibilityIdentifier: .addAccessMethodPortCell,
            leading: {
                itemFactory.leading(for: .generic(title: NSLocalizedString("Password", comment: "")))
            },
            trailing: {
                itemFactory.trailing(
                    for: .secureInput(
                        title: "",
                        placeholder: NSLocalizedString("Required", comment: ""),
                        text: passwordBinding)
                )
                .focused(focusedInput, equals: .password)
            },
            onSelect: {
                focusedInput.wrappedValue = .password
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
    }

    private var errorMessageView: some View {
        Group {
            ForEach(messages, id: \.self) { message in
                itemFactory.footer(for: .generic(text: message))
                    .padding(.leading, 16.0)
            }
        }
    }

}
