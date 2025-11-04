//
//  LogFileOutputStream.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

class LogFileOutputStream: TextOutputStream, @unchecked Sendable {
    private let queue = DispatchQueue(label: "LogFileOutputStreamQueue")

    private let baseFileURL: URL
    private var fileURL: URL
    private let encoding: String.Encoding
    private let maximumBufferCapacity: Int
    private let fileHeader: String
    private let fileSizeLimit: UInt64
    private let newLineChunkReadSize: Int

    /// Interval used for reopening the log file descriptor in the event of failure to open it in
    /// the first place, or when writing to it.
    private let reopenFileLogInterval: Duration

    private var state: State = .closed {
        didSet {
            switch (oldValue, state) {
            case (.opened, .waitingToReopen), (.closed, .waitingToReopen):
                startTimer()

            case (.waitingToReopen, .opened), (.waitingToReopen, .closed):
                stopTimer()

            default:
                break
            }
        }
    }

    /// Shorthand to get the file header in a `Data` writeable format
    private var headerData: Data { "\(fileHeader)\n".data(using: encoding, allowLossyConversion: true)! }

    private var timer: DispatchSourceTimer?
    private var buffer = Data()

    private enum State {
        case closed
        case opened(FileHandle)
        case waitingToReopen
    }

    init(
        fileURL: URL,
        header: String,
        fileSizeLimit: UInt64 = ApplicationConfiguration.logMaximumFileSize,
        encoding: String.Encoding = .utf8,
        maxBufferCapacity: Int = 16 * 1024,
        reopenFileLogInterval: Duration = .seconds(5),
        newLineChunkReadSize: Int = 35
    ) {
        self.fileURL = fileURL
        self.fileHeader = header
        self.fileSizeLimit = fileSizeLimit
        self.encoding = encoding
        self.maximumBufferCapacity = maxBufferCapacity
        self.reopenFileLogInterval = reopenFileLogInterval
        self.newLineChunkReadSize = newLineChunkReadSize

        baseFileURL = fileURL.deletingPathExtension()
    }

    deinit {
        stopTimer()
    }

    func write(_ string: String) {
        queue.async {
            self.writeOnQueue(string)
        }
    }

    /// Waits for write operations to finish by issuing a synchronous closure.
    /// - Note: This function is mainly used in unit tests to facilitate acting
    /// on disk writes. It should typically not be used in production code.
    func synchronize() {
        queue.sync {}
    }

    private func writeOnQueue(_ string: String) {
        dispatchPrecondition(condition: .onQueue(queue))
        guard let data = string.data(using: encoding) else { return }

        switch state {
        case .closed:
            do {
                let fileHandle = try openFileWithHeader(fileHeader)
                state = .opened(fileHandle)
                try write(fileHandle: fileHandle, data: data)
            } catch {
                bufferData(data)
                state = .waitingToReopen
            }

        case let .opened(fileHandle):
            do {
                try write(fileHandle: fileHandle, data: data)
            } catch {
                bufferData(data)
                state = .waitingToReopen
            }

        case .waitingToReopen:
            bufferData(data)
        }
    }

    private func write(fileHandle: FileHandle, data: Data) throws {
        dispatchPrecondition(condition: .onQueue(queue))

        let incomingDataSize = UInt64(data.count)

        // Make sure incoming data chunks are not larger than the file size limit.
        // Failure to handle this leads to data neither being written nor buffered/trimmed.
        guard incomingDataSize <= fileSizeLimit else {
            throw POSIXError(.EDQUOT)
        }

        let predictedFileSize = try fileHandle.offset() + incomingDataSize

        // Truncate file in half if threshold has been met, otherwise just write.
        if predictedFileSize >= fileSizeLimit {
            try truncateFileInHalf(fileHandle: fileHandle)
        }

        try fileHandle.write(contentsOf: data)
    }

    private func truncateFileInHalf(fileHandle: FileHandle) throws {
        let fileCenterOffset = UInt64(fileSizeLimit / 2)

        try fileHandle.seek(toOffset: fileCenterOffset)

        /// Advance the file offset to the next line (delimited by a \n) to make the log
        /// truncation appear more user friendly by not potentially cutting a log line in half
        try fileHandle.readUntilNextLineBreak(readSize: UInt64(newLineChunkReadSize), sizeLimit: fileSizeLimit)

        let fileLastHalf = fileHandle.availableData

        try fileHandle.truncate(atOffset: 0)
        try fileHandle.write(contentsOf: headerData)
        try fileHandle.write(contentsOf: fileLastHalf)
    }

    private func openFile() throws -> FileHandle {
        let oflag: Int32 = O_RDWR | O_CREAT
        let mode: mode_t = S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH

        let fd = fileURL.path.withCString { Darwin.open($0, oflag, mode) }

        if fd == -1 {
            let code = POSIXErrorCode(rawValue: errno)!
            throw POSIXError(code)
        } else {
            return FileHandle(fileDescriptor: fd, closeOnDealloc: true)
        }
    }

    private func startTimer() {
        timer?.cancel()

        let timer = DispatchSource.makeTimerSource(queue: queue)
        timer.setEventHandler { [weak self] in
            self?.reopenFile()
        }
        timer.schedule(
            wallDeadline: .now() + reopenFileLogInterval,
            repeating: reopenFileLogInterval.timeInterval
        )
        timer.activate()

        self.timer = timer
    }

    private func stopTimer() {
        timer?.cancel()
        timer = nil
    }

    private func openFileWithHeader(_ header: String) throws -> FileHandle {
        let fileHandle = try openFile()

        try write(fileHandle: fileHandle, data: headerData)

        return fileHandle
    }

    private func reopenFile() {
        do {
            // Write a message indicating that the file was reopened.
            let fileHandle =
                try openFileWithHeader("<Log file re-opened after failure. Buffered \(buffer.count) bytes of messages>")

            // Write all buffered messages.
            if !buffer.isEmpty {
                try write(fileHandle: fileHandle, data: buffer)
                buffer.removeAll()
            }

            state = .opened(fileHandle)
        } catch {
            state = .waitingToReopen
        }
    }

    private func bufferData(_ data: Data) {
        buffer.append(data)

        if buffer.count > maximumBufferCapacity {
            buffer.removeFirst(buffer.count - maximumBufferCapacity)
        }
    }
}

fileprivate extension FileHandle {
    /// Reads into the file until the next "\n" is reached.
    ///
    /// The file pointer will be set to the offset after the first "\n"
    /// character encountered. If the attempted read would go past
    /// the `sizeLimit` no reads are attempted and the file pointer
    /// will not be moved.
    ///
    func readUntilNextLineBreak(readSize: UInt64, sizeLimit: UInt64) throws {
        let currentOffset = try offset()
        // Ignore Integer overflow checks, files would not reach that size in the first place
        // Do not try to read past the end of the file
        guard currentOffset + readSize <= sizeLimit else { return }
        let readBytes = try read(upToCount: Int(readSize))

        // Find the first instance of the "\n" character
        if let newLineIndex = readBytes?.firstIndex(of: 10) {
            let offsetAfterNewLine = currentOffset + UInt64(newLineIndex) + 1
            try seek(toOffset: offsetAfterNewLine)
            return
        } else {
            // Keep reading until either a "\n" character, or the end of the file are found
            try readUntilNextLineBreak(readSize: readSize, sizeLimit: sizeLimit)
        }
    }
}
