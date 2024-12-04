//
//  LogFileOutputStream.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Interval used for reopening the log file descriptor in the event of failure to open it in
/// the first place, or when writing to it.
private let reopenFileLogInterval: Duration = .seconds(5)

class LogFileOutputStream: TextOutputStream, @unchecked Sendable {
    private let queue = DispatchQueue(label: "LogFileOutputStreamQueue", qos: .utility)

    private let baseFileURL: URL
    private var fileURL: URL
    private let encoding: String.Encoding
    private let maximumBufferCapacity: Int
    private let fileHeader: String
    private let fileSizeLimit: UInt64
    private var partialFileSizeCounter: UInt64 = 0
    private var partialFileNameCounter = 1

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
        maxBufferCapacity: Int = 16 * 1024
    ) {
        self.fileURL = fileURL
        self.fileHeader = header
        self.fileSizeLimit = fileSizeLimit
        self.encoding = encoding
        self.maximumBufferCapacity = maxBufferCapacity

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

        let predictedFileSize = partialFileSizeCounter + incomingDataSize

        // Rotate file if threshold has been met, then rerun the write operation.
        guard predictedFileSize <= fileSizeLimit else {
            try rotateFile(handle: fileHandle)
            write(String(data: data, encoding: encoding) ?? "")
            return
        }

        let bytesWritten = data.withUnsafeBytes { buffer -> Int in
            guard let ptr = buffer.baseAddress else { return 0 }

            return Darwin.write(fileHandle.fileDescriptor, ptr, buffer.count)
        }

        if bytesWritten == -1 {
            let code = POSIXErrorCode(rawValue: errno)!
            throw POSIXError(code)
        }

        partialFileSizeCounter += UInt64(bytesWritten)
    }

    private func rotateFile(handle: FileHandle) throws {
        try handle.close()

        state = .closed
        partialFileSizeCounter = 0
        fileURL = try incrementFileName()
    }

    private func openFile() throws -> FileHandle {
        let oflag: Int32 = O_WRONLY | O_CREAT
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

        let messageData =
            "\(header)\n"
                .data(using: encoding, allowLossyConversion: true)!
        try write(fileHandle: fileHandle, data: messageData)

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

    private func incrementFileName() throws -> URL {
        partialFileNameCounter += 1

        if let url = URL(string: baseFileURL.relativePath + "_\(partialFileNameCounter).log") {
            return url
        } else {
            throw POSIXError(.ENOENT)
        }
    }
}
