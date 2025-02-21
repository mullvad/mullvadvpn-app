//
//  LogFileOutputStreamTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2025-01-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadLogging
import Testing

@Suite("LogFileOutputStream Tests")
actor LogFileOutputStreamTests {
    let fileManager = FileManager.default
    var directoryPath: URL!

    init() async throws {
        directoryPath = FileManager.default.temporaryDirectory.appendingPathComponent(
            UUID().uuidString,
            isDirectory: true
        )

        try fileManager.createDirectory(
            at: directoryPath,
            withIntermediateDirectories: true
        )
    }

    deinit {
        try? fileManager.removeItem(at: directoryPath)
    }

    @Test func logHeaderGetsWrittenAtFileStartAfterTruncation() async throws {
        let header = "header"
        let message = """
        old

        """
        let fileSizeLimit: UInt64 = 20
        let fileURL = directoryPath.appendingPathComponent(UUID().uuidString)
        let stream = LogFileOutputStream(
            fileURL: fileURL,
            header: header,
            fileSizeLimit: fileSizeLimit,
            newLineChunkReadSize: 3
        )
        // Fill the file with the word "old" to force truncation in half
        for _ in 0 ..< 3 {
            stream.write(message)
        }
        /* At this point, the file contains the following string (of length 19)
         "header\nold\nold\nold"
                    ^
                    Half point of the file

         Writing the word "new" goes over the file size limit (20),
         so the file will get truncated to its half point.
         In order to keep a nice UX for reading log, the stream will move the internal file cursor to after the next "\n"
         character, and read the last half of the file in order to paste it at the beginning
         after truncation.
         In this example, the string "old\nold\n" will be buffered, which will then
         get prepended with "header\n"
          */
        stream.synchronize()
        stream.write("new")
        stream.synchronize()

        let fileContents = try #require(
            try String(contentsOf: fileURL, encoding: .utf8)
        )
        let expectedContents = """
        header
        old
        old
        new
        """

        #expect(fileContents == expectedContents)
    }

    @Test func fileSizeCounterGetsResetAfterTruncation() async throws {
        let header = "header"
        let message = """
        old

        """
        let fileSizeLimit: UInt64 = 20
        let fileURL = directoryPath.appendingPathComponent(UUID().uuidString)
        let stream = LogFileOutputStream(
            fileURL: fileURL,
            header: header,
            fileSizeLimit: fileSizeLimit
        )
        // Fill the file with the word "old" to force truncation in half
        for _ in 0 ..< 3 {
            stream.write(message)
        }
        // File gets truncated in half here
        stream.write("new")
        stream.write("a")
        stream.synchronize()

        /// If the `partialFileSizeCounter` didn't get reset after truncating,
        /// a new write will truncate the file again instead of just appending
        let expectedContents = """
        header
        d
        old
        newa
        """
        let fileContents = try #require(
            try String(contentsOf: fileURL, encoding: .utf8)
        )
        #expect(fileContents == expectedContents)
    }
}
