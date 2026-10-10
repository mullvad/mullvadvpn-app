// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import SwiftUI

struct LoginView: View {

    @State private var accountNumber: String = ""
    @State private var accountTextColor: Color = .white
    @Environment(Navigation.self) private var navigation: Navigation
    private var viewModel: LoggingIn

    init(viewModel: any LoggingIn) {
        self.viewModel = viewModel
    }

    var body: some View {
        if case .authenticating = viewModel.state {
            Text("Authenticating")
        } else {
            mainContent()
        }
    }

    @ViewBuilder
    func mainContent() -> some View {
        VStack(alignment: .leading) {
            Text("Login")
                .bold()
                .font(.title)
            Text("Enter your account number")
                .font(.headline)
                .foregroundStyle(accountTextColor)
            Form {
                TextField(
                    "Account", text: $accountNumber,
                    prompt: Text("0000 0000 0000 0000")
                        .foregroundStyle(.gray)
                )
                successLabel()
                    .task {
                        try? await Task.sleep(for: .seconds(1.5))
                        navigation.logIn()
                    }
                loginErrorLabel()
            }
            .onSubmit {
                Task {
                    await login()
                }
            }
        }
    }

    @ViewBuilder
    func successLabel() -> some View {
        if case .success = viewModel.state {
            Text("Login successful")
                .foregroundStyle(Color.mullvadSuccessColor)
        }
    }

    @ViewBuilder
    func loginErrorLabel() -> some View {
        if case let .failure(error) = viewModel.state {
            Text("\(error.description)")
                .foregroundStyle(Color.mullvadDangerColor)
        }
    }

    func login() async {
        print("Logging in with \(accountNumber)")
        if validateAccountNumber() {
            accountTextColor = .white
            await viewModel.loginWithNumber(accountNumber)
        } else {
            accountTextColor = .mullvadDangerColor
        }
    }

    func validateAccountNumber() -> Bool {
        accountNumber.allSatisfy({ $0.isNumber })
    }
}

#Preview("Initial state") {
    let stub = LoginViewModelStub()
    LoginView(viewModel: stub)
        .environment(Navigation())
}

#Preview("Success state") {
    let stub = LoginViewModelStub()
    LoginView(viewModel: stub)
        .environment(Navigation())
        .task {
            stub.state = .success
        }
}

#Preview("Authenticating state") {
    let stub = LoginViewModelStub()
    LoginView(viewModel: stub)
        .environment(Navigation())
        .task {
            stub.state = .authenticating
        }
}

#Preview("Error state") {
    let stub = LoginViewModelStub()
    LoginView(viewModel: stub)
        .environment(Navigation())
        .task {
            stub.state = .failure(POSIXError(.ENOSR))
        }
}
