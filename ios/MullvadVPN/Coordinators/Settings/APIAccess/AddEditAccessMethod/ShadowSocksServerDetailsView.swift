//// This Source Code Form is subject to the terms of the GPLv3 License.
//// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
////
//// This file incorporates work covered by the following copyright and
//// permission notice:
////
////   Copyright (c) Mullvad VPN AB. All rights reserved.
////
//// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

struct ShadowSocksServerDetailsView: View {
    let itemFactory = SegmentedListItemFactory()
    let viewModel: any ConfigureApiAccessMethodViewModelProtocol

    private var serverBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.shadowsocks.server },
            set: { viewModel.accessMethodViewModel.shadowsocks.server = $0 }
        )
    }

    private var portBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.shadowsocks.port },
            set: { viewModel.accessMethodViewModel.shadowsocks.port = $0 }
        )
    }

    private var passwordBinding: Binding<String> {
        Binding(
            get: { viewModel.accessMethodViewModel.shadowsocks.password },
            set: { viewModel.accessMethodViewModel.shadowsocks.password = $0 }
        )
    }

    private var cipherBinding: Binding<String> {
        Binding(
            get: {
                viewModel.accessMethodViewModel.shadowsocks.cipher.isEmpty
                    ? (viewModel.ciphers.first ?? "") : viewModel.accessMethodViewModel.shadowsocks.cipher
            },
            set: { viewModel.accessMethodViewModel.shadowsocks.cipher = $0 }
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
            passwordField
            cipherPicker
            errorMessageView
        }
        .onChange(of: viewModel.accessMethodViewModel.shadowsocks) {
            viewModel.clearErrors(for: [
                .server,
                .port,
                .password,
                .cipher,
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

    private var passwordField: some View {
        SegmentedListItem(
            isLastInList: false,
            accessibilityIdentifier: .addAccessMethodPasswordCell,
            borderStyle: .constant(viewModel.borderStyle(for: .password)),
            leading: {
                AccessMethodLabelView(title: "Password", textColor: viewModel.borderStyle(for: .password).textColor)
            },
            trailing: {
                ConfigurableTextField(
                    placeholder: LocalizedStringKey("Optional"),
                    text: passwordBinding,
                    borderStyle: .constant(.none),
                    appearance: InputViewAppearance(backgroundColor: .clear, showsClearButton: false)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
        .padding(.bottom, 1.0)
    }

    private var cipherPicker: some View {
        SegmentedListItem(
            isLastInList: true,
            accessibilityIdentifier: .addAccessMethodCipherCell,
            borderStyle: .constant(viewModel.borderStyle(for: .cipher)),
            leadingAndTrailingDestination: {
                ShadowsocksCipherPickerView(
                    viewModel: ShadowsocksCipherPickerViewModel(ciphers: viewModel.ciphers, current: cipherBinding)
                )
            },
            leading: {
                AccessMethodLabelView(title: "Cipher", textColor: viewModel.borderStyle(for: .cipher).textColor)
            },
            trailing: {
                itemFactory.trailing(
                    for: .drillDown(
                        title: cipherBinding.wrappedValue,
                        breadcrumb: viewModel.hasInvalidCipher ? .warning(.apiAccess) : nil)
                )
            }
        )
        .environment(\.isNestedInSegmentedListItem, true)
    }

    private var errorMessageView: some View {
        ValidationErrorView(error: messages.joined(separator: "\n"))
    }
}
