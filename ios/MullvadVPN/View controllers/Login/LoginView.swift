//
//  LoginView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-07-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct LoginView<ViewModel: LoginViewModel>: View {
    enum StatusActivityState {
        case hidden, activity, success, failure
    }

    @State var viewModel: ViewModel
    @State private var alert: MullvadAlert?
    @State private var buttonContainerVisible = true
    @State private var buttonContainerOpacity: CGFloat = 1
    @State private var textFieldOpacity: CGFloat = 1
    @FocusState private var isFocused: Bool

    var body: some View {
        GeometryReader { geometry in
            ScrollView {
                content
                    .padding(UIMetrics.contentInsets.toEdgeInsets)
                    .frame(minHeight: geometry.size.height, alignment: .center)
            }
        }
        .background(Color.mullvadBackground.ignoresSafeArea())
        .safeAreaInset(edge: .top) {
            if viewModel.showAccessMethodInvalidView {
                AccessMethodInvalidView {
                    viewModel.navigateToAccessMethods?()
                }
                .padding(UIMetrics.contentInsets.toEdgeInsets)
            }
        }
    }
}

// MARK: Content

extension LoginView {
    private var content: some View {
        let activityIconHeight: CGFloat = 60

        return VStack(spacing: 32) {
            textFieldContainer
            if buttonContainerVisible {
                buttonContainer
                    .opacity(buttonContainerOpacity)
                    .transition(.identity)
            }
        }
        .mullvadAlert(item: $alert)
        .overlay(alignment: .top) {
            statusActivityIcon
                .frame(width: activityIconHeight, height: activityIconHeight)
                .offset(y: -(activityIconHeight + 32))  // Height + spacing
        }
        .onChange(of: viewModel.showButtons) { _, newValue in
            animateButtonContainer(to: newValue)
        }
    }

    private var buttonContainer: some View {
        VStack(spacing: 20) {
            MullvadButton(text: "Log in", style: .success, action: viewModel.login)
                .disabled(!viewModel.enableLoginButton && viewModel.showButtons)
                .accessibilityIdentifier(.loginButton)

            HStack(spacing: 8) {
                Spacer()
                    .frame(height: 1)
                    .background(Color.MullvadOther.divider)
                Text("Or")
                    .font(.mullvadTiny)
                    .foregroundStyle(Color.mullvadTextPrimary)
                Spacer()
                    .frame(height: 1)
                    .background(Color.MullvadOther.divider)
            }

            MullvadButton(
                text: "Create new account",
                style: .secondary,
                action: {
                    if !viewModel.storedAccountNumber.isEmpty {
                        alert = createAccountAlert
                    } else {
                        viewModel.createAccount()
                    }
                }
            )
            .accessibilityIdentifier(.createAccountButton)
        }
    }

    private var textFieldContainer: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("Log in")
                .font(.mullvadBig)
                .foregroundStyle(Color.mullvadTextPrimary)

            ConfigurableTextField(
                title: "Account number",
                placeholder: "Enter your account number",
                text: $viewModel.accountNumber,
                isFocused: $isFocused,
                message: .constant(viewModel.errorMessage),
                borderStyle: .constant(viewModel.textFieldBorderStyle),
                configuration: .init(
                    formatter: GroupedTextFormatter.accountNumber,
                    autoComplete: .init(
                        suggestions: $viewModel.storedAccountNumber,
                        onSelect: { string in
                            viewModel.accountNumber = string
                            viewModel.login()
                        },
                        onRemove: { _ in
                            alert = removeSavedAccountAlert
                        }
                    ),
                    keyboardType: .numberPad,
                    submitConfiguration: .init(
                        label: .go,
                        action: {
                            viewModel.login()
                        }
                    ),
                )
            )
            .accessibilityIdentifier(.loginTextField)
            .disabled(viewModel.textFieldIsDisabled)
            .opacity(textFieldOpacity)
            .onAppear {
                isFocused = true
            }
        }
    }

    @ViewBuilder private var statusActivityIcon: some View {
        switch viewModel.statusActivityState {
        case .hidden:
            EmptyView()
        case .activity:
            ProgressView()
                .progressViewStyle(MullvadProgressViewStyle())
        case .failure:
            Image.mullvadIconFail
                .accessibilityIdentifier(.statusImageView)
                .accessibilityValue("fail")
        case .success:
            Image.mullvadIconSuccess
                .accessibilityIdentifier(.statusImageView)
        }
    }
}

// MARK: Alerts

extension LoginView {
    private var createAccountAlert: MullvadAlert {
        let message =
            "You already have a saved account number, by creating a new account "
            + "the saved account number will be removed from this device. This cannot be undone."

        return MullvadAlert(
            type: .warning,
            messages: [
                LocalizedStringKey(NSLocalizedString(message, comment: "")),
                "Do you want to create a new account?",
            ],
            actions: [
                .init(
                    type: .primary,
                    title: "Create new account",
                    identifier: .createAccountConfirmationButton,
                    handler: {
                        viewModel.createAccount()
                        alert = nil
                    },
                ),
                .init(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        alert = nil
                    }
                ),
            ]
        )
    }

    private var removeSavedAccountAlert: MullvadAlert {
        MullvadAlert(
            type: .warning,
            messages: [
                "Removing the saved account number from this device cannot be undone.",
                "Do you want to remove the saved account number?",
            ],
            actions: [
                .init(
                    type: .destructive,
                    title: "Remove",
                    handler: {
                        viewModel.storedAccountNumber = []
                        alert = nil
                    }
                ),
                .init(
                    type: .secondary,
                    title: "Cancel",
                    handler: {
                        alert = nil
                    }
                ),
            ]
        )
    }
}

// MARK: Helpers

extension LoginView {
    private func animateButtonContainer(to show: Bool) {
        Task { @MainActor in
            if show {
                withAnimation(.easeInOut(duration: 0.3)) {
                    buttonContainerVisible = true
                }
                try? await Task.sleep(for: .seconds(0.1))
                withAnimation(.easeInOut(duration: 0.3)) {
                    textFieldOpacity = 1
                    buttonContainerOpacity = 1
                }
            } else {
                withAnimation(.easeInOut(duration: 0.3)) {
                    textFieldOpacity = 0.2
                    buttonContainerOpacity = 0
                }
                try? await Task.sleep(for: .seconds(0.1))
                withAnimation(.easeInOut(duration: 0.3)) {
                    buttonContainerVisible = false
                }
            }
        }
    }
}

// MARK: Preview

private struct MockTunnelManager: LoginViewModelProviding {
    func setExistingAccount(accountNumber: String) async throws -> StoredAccountData {
        .init(identifier: "", number: "", expiry: .now)
    }

    func setNewAccount() async throws -> StoredAccountData {
        .init(identifier: "", number: "", expiry: .now)
    }
}

#Preview {
    LoginView(
        viewModel: LoginViewModel(
            interactor: LoginInteractor(
                tunnelManager: MockTunnelManager(),
                settingsManager: SettingsManager()
            ),
            loginState: .default
        )
    )
}
