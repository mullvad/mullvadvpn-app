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

struct SpeedTestView: View {

    @State private var speedTest = SpeedTest()

    @State private var isTesting = false
    @State private var downloadMbps: Double = 0
    @State private var progress: Double = 0
    @State private var errorMessage: String?

    var body: some View {

        VStack(spacing: 24) {

            Text("Internet Speed")
                .font(.title2)
                .fontWeight(.semibold)

            VStack(spacing: 4) {

                Text(
                    "\(downloadMbps, specifier: "%.0f")"
                )
                .font(
                    .system(
                        size: 64,
                        weight: .bold,
                        design: .rounded
                    )
                )

                Text("Mbps")
                    .font(.title3)
                    .foregroundStyle(.secondary)
            }

            if isTesting {

                ProgressView(value: progress)
                    .padding(.horizontal)

                Text("Testing download speed…")
                    .foregroundStyle(.secondary)

            } else {

                Button("Start Speed Test") {
                    startTest()
                }
                .buttonStyle(.borderedProminent)
            }

            if let errorMessage {

                Text(errorMessage)
                    .foregroundStyle(.red)
            }
        }
        .padding()
    }

    private func startTest() {

        isTesting = true
        downloadMbps = 0
        progress = 0
        errorMessage = nil

        Task {

            do {

                let result = try await speedTest.testDownload { progress in

                    Task { @MainActor in

                        downloadMbps = progress.downloadMbps
                        self.progress = progress.progress
                    }
                }

                await MainActor.run {

                    downloadMbps = result.downloadMbps
                    progress = 1
                    isTesting = false
                }

            } catch {

                await MainActor.run {

                    isTesting = false
                    errorMessage = error.localizedDescription
                }
            }
        }
    }
}


#Preview {
    SpeedTestView()
}
