/* SPDX-License-Identifier: Apache-2.0
 *
 * Copyright (C) 2017-2019 Jason A. Donenfeld <Jason@zx2c4.com>. All Rights Reserved.
 * Copyright (C) 2019 Amagicom AB. All Rights Reserved.
 */

package main

// #include <stdlib.h>
import "C"
import (
	"bufio"
	"bytes"
	"fmt"
	"io/ioutil"
	"log"
	"math"
	"net"
	"os"
	"os/signal"
	"runtime"
	"strings"
	"sync"
	"unsafe"

	"golang.org/x/sys/unix"

	"golang.zx2c4.com/wireguard/device"
	"golang.zx2c4.com/wireguard/ipc"
	"golang.zx2c4.com/wireguard/tun"
)

var (
	logFile     *os.File
	logFilePath *string
	logFileLock sync.RWMutex
)

type FileLogger struct{}

func (logger FileLogger) Write(buffer []byte) (int, error) {
	logFileLock.RLock()
	defer logFileLock.RUnlock()

	return logFile.Write(buffer)
}

type TunnelHandle struct {
	device *device.Device
	uapi   net.Listener
}

var tunnelHandles map[int32]TunnelHandle

func init() {
	device.RoamingDisabled = true
	tunnelHandles = make(map[int32]TunnelHandle)
	signals := make(chan os.Signal)
	signal.Notify(signals, unix.SIGUSR2)
	go func() {
		buf := make([]byte, os.Getpagesize())
		for {
			select {
			case <-signals:
				n := runtime.Stack(buf, true)
				buf[n] = 0
				log.Println("WireGuard/GoBackend/StackTrace - ", buf)
			}
		}
	}()
}

func newLogger(newLogFilePath string, level int) *device.Logger {
	openLogFile(newLogFilePath)

	logger := &device.Logger{
		Debug: newLogForLevel(device.LogLevelDebug, level),
		Info:  newLogForLevel(device.LogLevelInfo, level),
		Error: newLogForLevel(device.LogLevelError, level),
	}

	return logger
}

func openLogFile(newLogFilePath string) {
	logFileLock.Lock()
	defer logFileLock.Unlock()

	if logFilePath == nil || *logFilePath != newLogFilePath {
		if logFile != nil {
			logFile.Close()
		}

		logFilePath = &newLogFilePath
		backupLogFile(newLogFilePath)
		logFile, _ = os.Create(newLogFilePath)
	}
}

func backupLogFile(path string) {
	backupPath := fmt.Sprintf("%s.old.log", strings.TrimSuffix(path, ".log"))

	os.Rename(path, backupPath)
}

func newLogForLevel(level int, maxLevel int) *log.Logger {
	if level > maxLevel {
		return log.New(ioutil.Discard, "", log.Ldate|log.Ltime)
	}

	logger := &FileLogger{}

	prefix := ""
	switch level {
	case device.LogLevelDebug:
		prefix = "DEBUG: "
	case device.LogLevelInfo:
		prefix = "INFO: "
	case device.LogLevelError:
		prefix = "ERROR: "
	}

	return log.New(logger, prefix, log.Ldate|log.Ltime)
}

//export wgTurnOnWithFd
func wgTurnOnWithFd(cIfaceName *C.char, mtu int, cSettings *C.char, fd int, logFilePath *C.char, level int) int32 {

	logger := newLogger(C.GoString(logFilePath), level)
	if cIfaceName == nil {
		logger.Error.Println("cIfaceName is null")
		return -1
	}

	if cSettings == nil {
		logger.Error.Println("cSettings is null")
		return -1
	}
	settings := C.GoString(cSettings)
	ifaceName := C.GoString(cIfaceName)

	file := os.NewFile(uintptr(fd), "")
	tunDevice, err := tun.CreateTUNFromFile(file, mtu)
	if err != nil {
		logger.Error.Println(err)
		if err.Error() == "bad file descriptor" {
			return -2
		}
		return -1
	}

	device := device.NewDevice(tunDevice, logger)

	var uapi net.Listener

	uapiFile, err := ipc.UAPIOpen(ifaceName)
	if err != nil {
		logger.Error.Println(err)
	} else {
		uapi, err = ipc.UAPIListen(ifaceName, uapiFile)
		if err != nil {
			logger.Error.Println("Failed to start the UAPI")
			logger.Error.Println(err)
			uapiFile.Close()
		} else {
			go func() {
				for {
					conn, err := uapi.Accept()
					if err != nil {
						return
					}
					go device.IpcHandle(conn)
				}
			}()
		}
	}

	setError := device.IpcSetOperation(bufio.NewReader(strings.NewReader(settings)))
	if setError != nil {
		logger.Error.Println(setError)
		device.Close()
		return -2
	}
	var i int32
	for i = 0; i < math.MaxInt32; i++ {
		if _, exists := tunnelHandles[i]; !exists {
			break
		}
	}
	if i == math.MaxInt32 {
		device.Close()
		return -1
	}
	tunnelHandles[i] = TunnelHandle{device: device, uapi: uapi}
	device.Up()
	return i
}

//export wgGetConfig
func wgGetConfig(index int32) *C.char {
	handle, ok := tunnelHandles[index]
	if !ok {
		return nil
	}

	settings := new(bytes.Buffer)
	writer := bufio.NewWriter(settings)
	if err := handle.device.IpcGetOperation(writer); err != nil {
		return nil
	}
	writer.Flush()
	return C.CString(settings.String())
}

//export wgFreePtr
func wgFreePtr(ptr unsafe.Pointer) {
		C.free(ptr)
}


//export wgTurnOff
func wgTurnOff(tunnelHandle int32) {
	handle, ok := tunnelHandles[tunnelHandle]
	if !ok {
		return
	}
	delete(tunnelHandles, tunnelHandle)
	if handle.uapi != nil {
		handle.uapi.Close()
	}
	handle.device.Close()
}

//export wgVersion
func wgVersion() *C.char {
	return C.CString(device.WireGuardGoVersion)
}

func main() {}
