// package: mullvad_daemon.management_interface
// file: management_interface.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";
import * as google_protobuf_empty_pb from "google-protobuf/google/protobuf/empty_pb";
import * as google_protobuf_timestamp_pb from "google-protobuf/google/protobuf/timestamp_pb";
import * as google_protobuf_wrappers_pb from "google-protobuf/google/protobuf/wrappers_pb";
import * as google_protobuf_duration_pb from "google-protobuf/google/protobuf/duration_pb";

export class AppUpgradeEvent extends jspb.Message { 

    hasDownloadStarting(): boolean;
    clearDownloadStarting(): void;
    getDownloadStarting(): AppUpgradeDownloadStarting | undefined;
    setDownloadStarting(value?: AppUpgradeDownloadStarting): AppUpgradeEvent;

    hasDownloadProgress(): boolean;
    clearDownloadProgress(): void;
    getDownloadProgress(): AppUpgradeDownloadProgress | undefined;
    setDownloadProgress(value?: AppUpgradeDownloadProgress): AppUpgradeEvent;

    hasUpgradeAborted(): boolean;
    clearUpgradeAborted(): void;
    getUpgradeAborted(): AppUpgradeAborted | undefined;
    setUpgradeAborted(value?: AppUpgradeAborted): AppUpgradeEvent;

    hasVerifyingInstaller(): boolean;
    clearVerifyingInstaller(): void;
    getVerifyingInstaller(): AppUpgradeVerifyingInstaller | undefined;
    setVerifyingInstaller(value?: AppUpgradeVerifyingInstaller): AppUpgradeEvent;

    hasVerifiedInstaller(): boolean;
    clearVerifiedInstaller(): void;
    getVerifiedInstaller(): AppUpgradeVerifiedInstaller | undefined;
    setVerifiedInstaller(value?: AppUpgradeVerifiedInstaller): AppUpgradeEvent;

    hasError(): boolean;
    clearError(): void;
    getError(): AppUpgradeError | undefined;
    setError(value?: AppUpgradeError): AppUpgradeEvent;

    getEventCase(): AppUpgradeEvent.EventCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeEvent.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeEvent): AppUpgradeEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeEvent;
    static deserializeBinaryFromReader(message: AppUpgradeEvent, reader: jspb.BinaryReader): AppUpgradeEvent;
}

export namespace AppUpgradeEvent {
    export type AsObject = {
        downloadStarting?: AppUpgradeDownloadStarting.AsObject,
        downloadProgress?: AppUpgradeDownloadProgress.AsObject,
        upgradeAborted?: AppUpgradeAborted.AsObject,
        verifyingInstaller?: AppUpgradeVerifyingInstaller.AsObject,
        verifiedInstaller?: AppUpgradeVerifiedInstaller.AsObject,
        error?: AppUpgradeError.AsObject,
    }

    export enum EventCase {
        EVENT_NOT_SET = 0,
        DOWNLOAD_STARTING = 1,
        DOWNLOAD_PROGRESS = 2,
        UPGRADE_ABORTED = 3,
        VERIFYING_INSTALLER = 4,
        VERIFIED_INSTALLER = 5,
        ERROR = 6,
    }

}

export class AppUpgradeDownloadStarting extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeDownloadStarting.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeDownloadStarting): AppUpgradeDownloadStarting.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeDownloadStarting, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeDownloadStarting;
    static deserializeBinaryFromReader(message: AppUpgradeDownloadStarting, reader: jspb.BinaryReader): AppUpgradeDownloadStarting;
}

export namespace AppUpgradeDownloadStarting {
    export type AsObject = {
    }
}

export class AppUpgradeDownloadProgress extends jspb.Message { 
    getServer(): string;
    setServer(value: string): AppUpgradeDownloadProgress;
    getProgress(): number;
    setProgress(value: number): AppUpgradeDownloadProgress;

    hasTimeLeft(): boolean;
    clearTimeLeft(): void;
    getTimeLeft(): google_protobuf_duration_pb.Duration | undefined;
    setTimeLeft(value?: google_protobuf_duration_pb.Duration): AppUpgradeDownloadProgress;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeDownloadProgress.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeDownloadProgress): AppUpgradeDownloadProgress.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeDownloadProgress, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeDownloadProgress;
    static deserializeBinaryFromReader(message: AppUpgradeDownloadProgress, reader: jspb.BinaryReader): AppUpgradeDownloadProgress;
}

export namespace AppUpgradeDownloadProgress {
    export type AsObject = {
        server: string,
        progress: number,
        timeLeft?: google_protobuf_duration_pb.Duration.AsObject,
    }
}

export class AppUpgradeAborted extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeAborted.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeAborted): AppUpgradeAborted.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeAborted, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeAborted;
    static deserializeBinaryFromReader(message: AppUpgradeAborted, reader: jspb.BinaryReader): AppUpgradeAborted;
}

export namespace AppUpgradeAborted {
    export type AsObject = {
    }
}

export class AppUpgradeVerifyingInstaller extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeVerifyingInstaller.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeVerifyingInstaller): AppUpgradeVerifyingInstaller.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeVerifyingInstaller, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeVerifyingInstaller;
    static deserializeBinaryFromReader(message: AppUpgradeVerifyingInstaller, reader: jspb.BinaryReader): AppUpgradeVerifyingInstaller;
}

export namespace AppUpgradeVerifyingInstaller {
    export type AsObject = {
    }
}

export class AppUpgradeVerifiedInstaller extends jspb.Message { 

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeVerifiedInstaller.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeVerifiedInstaller): AppUpgradeVerifiedInstaller.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeVerifiedInstaller, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeVerifiedInstaller;
    static deserializeBinaryFromReader(message: AppUpgradeVerifiedInstaller, reader: jspb.BinaryReader): AppUpgradeVerifiedInstaller;
}

export namespace AppUpgradeVerifiedInstaller {
    export type AsObject = {
    }
}

export class AppUpgradeError extends jspb.Message { 
    getError(): AppUpgradeError.Error;
    setError(value: AppUpgradeError.Error): AppUpgradeError;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppUpgradeError.AsObject;
    static toObject(includeInstance: boolean, msg: AppUpgradeError): AppUpgradeError.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppUpgradeError, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppUpgradeError;
    static deserializeBinaryFromReader(message: AppUpgradeError, reader: jspb.BinaryReader): AppUpgradeError;
}

export namespace AppUpgradeError {
    export type AsObject = {
        error: AppUpgradeError.Error,
    }

    export enum Error {
    GENERAL_ERROR = 0,
    DOWNLOAD_FAILED = 1,
    VERIFICATION_FAILED = 2,
    }

}

export class Seed extends jspb.Message { 
    getSeed(): number;
    setSeed(value: number): Seed;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Seed.AsObject;
    static toObject(includeInstance: boolean, msg: Seed): Seed.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Seed, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Seed;
    static deserializeBinaryFromReader(message: Seed, reader: jspb.BinaryReader): Seed;
}

export namespace Seed {
    export type AsObject = {
        seed: number,
    }
}

export class Rollout extends jspb.Message { 
    getThreshold(): number;
    setThreshold(value: number): Rollout;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Rollout.AsObject;
    static toObject(includeInstance: boolean, msg: Rollout): Rollout.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Rollout, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Rollout;
    static deserializeBinaryFromReader(message: Rollout, reader: jspb.BinaryReader): Rollout;
}

export namespace Rollout {
    export type AsObject = {
        threshold: number,
    }
}

export class UUID extends jspb.Message { 
    getValue(): string;
    setValue(value: string): UUID;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): UUID.AsObject;
    static toObject(includeInstance: boolean, msg: UUID): UUID.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: UUID, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): UUID;
    static deserializeBinaryFromReader(message: UUID, reader: jspb.BinaryReader): UUID;
}

export namespace UUID {
    export type AsObject = {
        value: string,
    }
}

export class AccountData extends jspb.Message { 
    getId(): string;
    setId(value: string): AccountData;

    hasExpiry(): boolean;
    clearExpiry(): void;
    getExpiry(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setExpiry(value?: google_protobuf_timestamp_pb.Timestamp): AccountData;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccountData.AsObject;
    static toObject(includeInstance: boolean, msg: AccountData): AccountData.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccountData, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccountData;
    static deserializeBinaryFromReader(message: AccountData, reader: jspb.BinaryReader): AccountData;
}

export namespace AccountData {
    export type AsObject = {
        id: string,
        expiry?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    }
}

export class AccountHistory extends jspb.Message { 

    hasNumber(): boolean;
    clearNumber(): void;
    getNumber(): google_protobuf_wrappers_pb.StringValue | undefined;
    setNumber(value?: google_protobuf_wrappers_pb.StringValue): AccountHistory;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccountHistory.AsObject;
    static toObject(includeInstance: boolean, msg: AccountHistory): AccountHistory.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccountHistory, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccountHistory;
    static deserializeBinaryFromReader(message: AccountHistory, reader: jspb.BinaryReader): AccountHistory;
}

export namespace AccountHistory {
    export type AsObject = {
        number?: google_protobuf_wrappers_pb.StringValue.AsObject,
    }
}

export class VoucherSubmission extends jspb.Message { 
    getSecondsAdded(): number;
    setSecondsAdded(value: number): VoucherSubmission;

    hasNewExpiry(): boolean;
    clearNewExpiry(): void;
    getNewExpiry(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setNewExpiry(value?: google_protobuf_timestamp_pb.Timestamp): VoucherSubmission;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VoucherSubmission.AsObject;
    static toObject(includeInstance: boolean, msg: VoucherSubmission): VoucherSubmission.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VoucherSubmission, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VoucherSubmission;
    static deserializeBinaryFromReader(message: VoucherSubmission, reader: jspb.BinaryReader): VoucherSubmission;
}

export namespace VoucherSubmission {
    export type AsObject = {
        secondsAdded: number,
        newExpiry?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    }
}

export class ErrorState extends jspb.Message { 
    getCause(): ErrorState.Cause;
    setCause(value: ErrorState.Cause): ErrorState;

    hasBlockingError(): boolean;
    clearBlockingError(): void;
    getBlockingError(): ErrorState.FirewallPolicyError | undefined;
    setBlockingError(value?: ErrorState.FirewallPolicyError): ErrorState;
    getAuthFailedError(): ErrorState.AuthFailedError;
    setAuthFailedError(value: ErrorState.AuthFailedError): ErrorState;
    getParameterError(): ErrorState.GenerationError;
    setParameterError(value: ErrorState.GenerationError): ErrorState;

    hasPolicyError(): boolean;
    clearPolicyError(): void;
    getPolicyError(): ErrorState.FirewallPolicyError | undefined;
    setPolicyError(value?: ErrorState.FirewallPolicyError): ErrorState;

    hasCreateTunnelError(): boolean;
    clearCreateTunnelError(): void;
    getCreateTunnelError(): number | undefined;
    setCreateTunnelError(value: number): ErrorState;

    hasOtherAlwaysOnAppError(): boolean;
    clearOtherAlwaysOnAppError(): void;
    getOtherAlwaysOnAppError(): ErrorState.OtherAlwaysOnAppError | undefined;
    setOtherAlwaysOnAppError(value?: ErrorState.OtherAlwaysOnAppError): ErrorState;

    hasInvalidDnsServersError(): boolean;
    clearInvalidDnsServersError(): void;
    getInvalidDnsServersError(): ErrorState.InvalidDnsServersError | undefined;
    setInvalidDnsServersError(value?: ErrorState.InvalidDnsServersError): ErrorState;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ErrorState.AsObject;
    static toObject(includeInstance: boolean, msg: ErrorState): ErrorState.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ErrorState, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ErrorState;
    static deserializeBinaryFromReader(message: ErrorState, reader: jspb.BinaryReader): ErrorState;
}

export namespace ErrorState {
    export type AsObject = {
        cause: ErrorState.Cause,
        blockingError?: ErrorState.FirewallPolicyError.AsObject,
        authFailedError: ErrorState.AuthFailedError,
        parameterError: ErrorState.GenerationError,
        policyError?: ErrorState.FirewallPolicyError.AsObject,
        createTunnelError?: number,
        otherAlwaysOnAppError?: ErrorState.OtherAlwaysOnAppError.AsObject,
        invalidDnsServersError?: ErrorState.InvalidDnsServersError.AsObject,
    }


    export class FirewallPolicyError extends jspb.Message { 
        getType(): ErrorState.FirewallPolicyError.ErrorType;
        setType(value: ErrorState.FirewallPolicyError.ErrorType): FirewallPolicyError;
        getLockPid(): number;
        setLockPid(value: number): FirewallPolicyError;

        hasLockName(): boolean;
        clearLockName(): void;
        getLockName(): string | undefined;
        setLockName(value: string): FirewallPolicyError;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): FirewallPolicyError.AsObject;
        static toObject(includeInstance: boolean, msg: FirewallPolicyError): FirewallPolicyError.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: FirewallPolicyError, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): FirewallPolicyError;
        static deserializeBinaryFromReader(message: FirewallPolicyError, reader: jspb.BinaryReader): FirewallPolicyError;
    }

    export namespace FirewallPolicyError {
        export type AsObject = {
            type: ErrorState.FirewallPolicyError.ErrorType,
            lockPid: number,
            lockName?: string,
        }

        export enum ErrorType {
    GENERIC = 0,
    LOCKED = 1,
        }

    }

    export class OtherAlwaysOnAppError extends jspb.Message { 
        getAppName(): string;
        setAppName(value: string): OtherAlwaysOnAppError;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): OtherAlwaysOnAppError.AsObject;
        static toObject(includeInstance: boolean, msg: OtherAlwaysOnAppError): OtherAlwaysOnAppError.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: OtherAlwaysOnAppError, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): OtherAlwaysOnAppError;
        static deserializeBinaryFromReader(message: OtherAlwaysOnAppError, reader: jspb.BinaryReader): OtherAlwaysOnAppError;
    }

    export namespace OtherAlwaysOnAppError {
        export type AsObject = {
            appName: string,
        }
    }

    export class InvalidDnsServersError extends jspb.Message { 
        clearIpAddrsList(): void;
        getIpAddrsList(): Array<string>;
        setIpAddrsList(value: Array<string>): InvalidDnsServersError;
        addIpAddrs(value: string, index?: number): string;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): InvalidDnsServersError.AsObject;
        static toObject(includeInstance: boolean, msg: InvalidDnsServersError): InvalidDnsServersError.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: InvalidDnsServersError, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): InvalidDnsServersError;
        static deserializeBinaryFromReader(message: InvalidDnsServersError, reader: jspb.BinaryReader): InvalidDnsServersError;
    }

    export namespace InvalidDnsServersError {
        export type AsObject = {
            ipAddrsList: Array<string>,
        }
    }


    export enum Cause {
    AUTH_FAILED = 0,
    IPV6_UNAVAILABLE = 1,
    SET_FIREWALL_POLICY_ERROR = 2,
    SET_DNS_ERROR = 3,
    START_TUNNEL_ERROR = 4,
    CREATE_TUNNEL_DEVICE = 5,
    TUNNEL_PARAMETER_ERROR = 6,
    IS_OFFLINE = 7,
    NOT_PREPARED = 8,
    OTHER_ALWAYS_ON_APP = 9,
    OTHER_LEGACY_ALWAYS_ON_VPN = 10,
    INVALID_DNS_SERVERS = 11,
    SPLIT_TUNNEL_ERROR = 12,
    NEED_FULL_DISK_PERMISSIONS = 13,
    }

    export enum AuthFailedError {
    UNKNOWN = 0,
    INVALID_ACCOUNT = 1,
    EXPIRED_ACCOUNT = 2,
    TOO_MANY_CONNECTIONS = 3,
    }

    export enum GenerationError {
    NO_MATCHING_RELAY_ENTRY = 0,
    NO_MATCHING_RELAY_EXIT = 1,
    NO_MATCHING_RELAY = 2,
    NO_MATCHING_BRIDGE_RELAY = 3,
    CUSTOM_TUNNEL_HOST_RESOLUTION_ERROR = 4,
    NETWORK_IPV4_UNAVAILABLE = 5,
    NETWORK_IPV6_UNAVAILABLE = 6,
    }

}

export class TunnelState extends jspb.Message { 

    hasDisconnected(): boolean;
    clearDisconnected(): void;
    getDisconnected(): TunnelState.Disconnected | undefined;
    setDisconnected(value?: TunnelState.Disconnected): TunnelState;

    hasConnecting(): boolean;
    clearConnecting(): void;
    getConnecting(): TunnelState.Connecting | undefined;
    setConnecting(value?: TunnelState.Connecting): TunnelState;

    hasConnected(): boolean;
    clearConnected(): void;
    getConnected(): TunnelState.Connected | undefined;
    setConnected(value?: TunnelState.Connected): TunnelState;

    hasDisconnecting(): boolean;
    clearDisconnecting(): void;
    getDisconnecting(): TunnelState.Disconnecting | undefined;
    setDisconnecting(value?: TunnelState.Disconnecting): TunnelState;

    hasError(): boolean;
    clearError(): void;
    getError(): TunnelState.Error | undefined;
    setError(value?: TunnelState.Error): TunnelState;

    getStateCase(): TunnelState.StateCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TunnelState.AsObject;
    static toObject(includeInstance: boolean, msg: TunnelState): TunnelState.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TunnelState, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TunnelState;
    static deserializeBinaryFromReader(message: TunnelState, reader: jspb.BinaryReader): TunnelState;
}

export namespace TunnelState {
    export type AsObject = {
        disconnected?: TunnelState.Disconnected.AsObject,
        connecting?: TunnelState.Connecting.AsObject,
        connected?: TunnelState.Connected.AsObject,
        disconnecting?: TunnelState.Disconnecting.AsObject,
        error?: TunnelState.Error.AsObject,
    }


    export class Disconnected extends jspb.Message { 

        hasDisconnectedLocation(): boolean;
        clearDisconnectedLocation(): void;
        getDisconnectedLocation(): GeoIpLocation | undefined;
        setDisconnectedLocation(value?: GeoIpLocation): Disconnected;
        getLockedDown(): boolean;
        setLockedDown(value: boolean): Disconnected;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Disconnected.AsObject;
        static toObject(includeInstance: boolean, msg: Disconnected): Disconnected.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Disconnected, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Disconnected;
        static deserializeBinaryFromReader(message: Disconnected, reader: jspb.BinaryReader): Disconnected;
    }

    export namespace Disconnected {
        export type AsObject = {
            disconnectedLocation?: GeoIpLocation.AsObject,
            lockedDown: boolean,
        }
    }

    export class Connecting extends jspb.Message { 

        hasRelayInfo(): boolean;
        clearRelayInfo(): void;
        getRelayInfo(): TunnelStateRelayInfo | undefined;
        setRelayInfo(value?: TunnelStateRelayInfo): Connecting;

        hasFeatureIndicators(): boolean;
        clearFeatureIndicators(): void;
        getFeatureIndicators(): FeatureIndicators | undefined;
        setFeatureIndicators(value?: FeatureIndicators): Connecting;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Connecting.AsObject;
        static toObject(includeInstance: boolean, msg: Connecting): Connecting.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Connecting, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Connecting;
        static deserializeBinaryFromReader(message: Connecting, reader: jspb.BinaryReader): Connecting;
    }

    export namespace Connecting {
        export type AsObject = {
            relayInfo?: TunnelStateRelayInfo.AsObject,
            featureIndicators?: FeatureIndicators.AsObject,
        }
    }

    export class Connected extends jspb.Message { 

        hasRelayInfo(): boolean;
        clearRelayInfo(): void;
        getRelayInfo(): TunnelStateRelayInfo | undefined;
        setRelayInfo(value?: TunnelStateRelayInfo): Connected;

        hasFeatureIndicators(): boolean;
        clearFeatureIndicators(): void;
        getFeatureIndicators(): FeatureIndicators | undefined;
        setFeatureIndicators(value?: FeatureIndicators): Connected;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Connected.AsObject;
        static toObject(includeInstance: boolean, msg: Connected): Connected.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Connected, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Connected;
        static deserializeBinaryFromReader(message: Connected, reader: jspb.BinaryReader): Connected;
    }

    export namespace Connected {
        export type AsObject = {
            relayInfo?: TunnelStateRelayInfo.AsObject,
            featureIndicators?: FeatureIndicators.AsObject,
        }
    }

    export class Disconnecting extends jspb.Message { 
        getAfterDisconnect(): AfterDisconnect;
        setAfterDisconnect(value: AfterDisconnect): Disconnecting;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Disconnecting.AsObject;
        static toObject(includeInstance: boolean, msg: Disconnecting): Disconnecting.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Disconnecting, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Disconnecting;
        static deserializeBinaryFromReader(message: Disconnecting, reader: jspb.BinaryReader): Disconnecting;
    }

    export namespace Disconnecting {
        export type AsObject = {
            afterDisconnect: AfterDisconnect,
        }
    }

    export class Error extends jspb.Message { 

        hasErrorState(): boolean;
        clearErrorState(): void;
        getErrorState(): ErrorState | undefined;
        setErrorState(value?: ErrorState): Error;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Error.AsObject;
        static toObject(includeInstance: boolean, msg: Error): Error.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Error, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Error;
        static deserializeBinaryFromReader(message: Error, reader: jspb.BinaryReader): Error;
    }

    export namespace Error {
        export type AsObject = {
            errorState?: ErrorState.AsObject,
        }
    }


    export enum StateCase {
        STATE_NOT_SET = 0,
        DISCONNECTED = 1,
        CONNECTING = 2,
        CONNECTED = 3,
        DISCONNECTING = 4,
        ERROR = 5,
    }

}

export class TunnelStateRelayInfo extends jspb.Message { 

    hasTunnelEndpoint(): boolean;
    clearTunnelEndpoint(): void;
    getTunnelEndpoint(): TunnelEndpoint | undefined;
    setTunnelEndpoint(value?: TunnelEndpoint): TunnelStateRelayInfo;

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): GeoIpLocation | undefined;
    setLocation(value?: GeoIpLocation): TunnelStateRelayInfo;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TunnelStateRelayInfo.AsObject;
    static toObject(includeInstance: boolean, msg: TunnelStateRelayInfo): TunnelStateRelayInfo.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TunnelStateRelayInfo, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TunnelStateRelayInfo;
    static deserializeBinaryFromReader(message: TunnelStateRelayInfo, reader: jspb.BinaryReader): TunnelStateRelayInfo;
}

export namespace TunnelStateRelayInfo {
    export type AsObject = {
        tunnelEndpoint?: TunnelEndpoint.AsObject,
        location?: GeoIpLocation.AsObject,
    }
}

export class TunnelEndpoint extends jspb.Message { 
    getAddress(): string;
    setAddress(value: string): TunnelEndpoint;
    getProtocol(): TransportProtocol;
    setProtocol(value: TransportProtocol): TunnelEndpoint;
    getQuantumResistant(): boolean;
    setQuantumResistant(value: boolean): TunnelEndpoint;

    hasObfuscation(): boolean;
    clearObfuscation(): void;
    getObfuscation(): ObfuscationInfo | undefined;
    setObfuscation(value?: ObfuscationInfo): TunnelEndpoint;

    hasEntryEndpoint(): boolean;
    clearEntryEndpoint(): void;
    getEntryEndpoint(): Endpoint | undefined;
    setEntryEndpoint(value?: Endpoint): TunnelEndpoint;

    hasTunnelMetadata(): boolean;
    clearTunnelMetadata(): void;
    getTunnelMetadata(): TunnelMetadata | undefined;
    setTunnelMetadata(value?: TunnelMetadata): TunnelEndpoint;
    getDaita(): boolean;
    setDaita(value: boolean): TunnelEndpoint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TunnelEndpoint.AsObject;
    static toObject(includeInstance: boolean, msg: TunnelEndpoint): TunnelEndpoint.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TunnelEndpoint, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TunnelEndpoint;
    static deserializeBinaryFromReader(message: TunnelEndpoint, reader: jspb.BinaryReader): TunnelEndpoint;
}

export namespace TunnelEndpoint {
    export type AsObject = {
        address: string,
        protocol: TransportProtocol,
        quantumResistant: boolean,
        obfuscation?: ObfuscationInfo.AsObject,
        entryEndpoint?: Endpoint.AsObject,
        tunnelMetadata?: TunnelMetadata.AsObject,
        daita: boolean,
    }
}

export class FeatureIndicators extends jspb.Message { 
    clearActiveFeaturesList(): void;
    getActiveFeaturesList(): Array<FeatureIndicator>;
    setActiveFeaturesList(value: Array<FeatureIndicator>): FeatureIndicators;
    addActiveFeatures(value: FeatureIndicator, index?: number): FeatureIndicator;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): FeatureIndicators.AsObject;
    static toObject(includeInstance: boolean, msg: FeatureIndicators): FeatureIndicators.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: FeatureIndicators, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): FeatureIndicators;
    static deserializeBinaryFromReader(message: FeatureIndicators, reader: jspb.BinaryReader): FeatureIndicators;
}

export namespace FeatureIndicators {
    export type AsObject = {
        activeFeaturesList: Array<FeatureIndicator>,
    }
}

export class ObfuscationInfo extends jspb.Message { 

    hasSingle(): boolean;
    clearSingle(): void;
    getSingle(): ObfuscationEndpoint | undefined;
    setSingle(value?: ObfuscationEndpoint): ObfuscationInfo;

    hasMultiple(): boolean;
    clearMultiple(): void;
    getMultiple(): MultiplexObfuscation | undefined;
    setMultiple(value?: MultiplexObfuscation): ObfuscationInfo;

    getTypeCase(): ObfuscationInfo.TypeCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ObfuscationInfo.AsObject;
    static toObject(includeInstance: boolean, msg: ObfuscationInfo): ObfuscationInfo.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ObfuscationInfo, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ObfuscationInfo;
    static deserializeBinaryFromReader(message: ObfuscationInfo, reader: jspb.BinaryReader): ObfuscationInfo;
}

export namespace ObfuscationInfo {
    export type AsObject = {
        single?: ObfuscationEndpoint.AsObject,
        multiple?: MultiplexObfuscation.AsObject,
    }

    export enum TypeCase {
        TYPE_NOT_SET = 0,
        SINGLE = 1,
        MULTIPLE = 2,
    }

}

export class MultiplexObfuscation extends jspb.Message { 

    hasDirect(): boolean;
    clearDirect(): void;
    getDirect(): Endpoint | undefined;
    setDirect(value?: Endpoint): MultiplexObfuscation;
    clearObfuscatorsList(): void;
    getObfuscatorsList(): Array<ObfuscationEndpoint>;
    setObfuscatorsList(value: Array<ObfuscationEndpoint>): MultiplexObfuscation;
    addObfuscators(value?: ObfuscationEndpoint, index?: number): ObfuscationEndpoint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): MultiplexObfuscation.AsObject;
    static toObject(includeInstance: boolean, msg: MultiplexObfuscation): MultiplexObfuscation.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: MultiplexObfuscation, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): MultiplexObfuscation;
    static deserializeBinaryFromReader(message: MultiplexObfuscation, reader: jspb.BinaryReader): MultiplexObfuscation;
}

export namespace MultiplexObfuscation {
    export type AsObject = {
        direct?: Endpoint.AsObject,
        obfuscatorsList: Array<ObfuscationEndpoint.AsObject>,
    }
}

export class ObfuscationEndpoint extends jspb.Message { 

    hasEndpoint(): boolean;
    clearEndpoint(): void;
    getEndpoint(): Endpoint | undefined;
    setEndpoint(value?: Endpoint): ObfuscationEndpoint;
    getObfuscationType(): ObfuscationEndpoint.ObfuscationType;
    setObfuscationType(value: ObfuscationEndpoint.ObfuscationType): ObfuscationEndpoint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ObfuscationEndpoint.AsObject;
    static toObject(includeInstance: boolean, msg: ObfuscationEndpoint): ObfuscationEndpoint.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ObfuscationEndpoint, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ObfuscationEndpoint;
    static deserializeBinaryFromReader(message: ObfuscationEndpoint, reader: jspb.BinaryReader): ObfuscationEndpoint;
}

export namespace ObfuscationEndpoint {
    export type AsObject = {
        endpoint?: Endpoint.AsObject,
        obfuscationType: ObfuscationEndpoint.ObfuscationType,
    }

    export enum ObfuscationType {
    UDP2TCP = 0,
    SHADOWSOCKS = 1,
    QUIC = 2,
    LWO = 3,
    }

}

export class Endpoint extends jspb.Message { 
    getAddress(): string;
    setAddress(value: string): Endpoint;
    getProtocol(): TransportProtocol;
    setProtocol(value: TransportProtocol): Endpoint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Endpoint.AsObject;
    static toObject(includeInstance: boolean, msg: Endpoint): Endpoint.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Endpoint, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Endpoint;
    static deserializeBinaryFromReader(message: Endpoint, reader: jspb.BinaryReader): Endpoint;
}

export namespace Endpoint {
    export type AsObject = {
        address: string,
        protocol: TransportProtocol,
    }
}

export class GeoIpLocation extends jspb.Message { 

    hasIpv4(): boolean;
    clearIpv4(): void;
    getIpv4(): string | undefined;
    setIpv4(value: string): GeoIpLocation;

    hasIpv6(): boolean;
    clearIpv6(): void;
    getIpv6(): string | undefined;
    setIpv6(value: string): GeoIpLocation;
    getCountry(): string;
    setCountry(value: string): GeoIpLocation;

    hasCity(): boolean;
    clearCity(): void;
    getCity(): string | undefined;
    setCity(value: string): GeoIpLocation;
    getLatitude(): number;
    setLatitude(value: number): GeoIpLocation;
    getLongitude(): number;
    setLongitude(value: number): GeoIpLocation;
    getMullvadExitIp(): boolean;
    setMullvadExitIp(value: boolean): GeoIpLocation;

    hasHostname(): boolean;
    clearHostname(): void;
    getHostname(): string | undefined;
    setHostname(value: string): GeoIpLocation;

    hasEntryHostname(): boolean;
    clearEntryHostname(): void;
    getEntryHostname(): string | undefined;
    setEntryHostname(value: string): GeoIpLocation;

    hasObfuscatorHostname(): boolean;
    clearObfuscatorHostname(): void;
    getObfuscatorHostname(): string | undefined;
    setObfuscatorHostname(value: string): GeoIpLocation;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GeoIpLocation.AsObject;
    static toObject(includeInstance: boolean, msg: GeoIpLocation): GeoIpLocation.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GeoIpLocation, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GeoIpLocation;
    static deserializeBinaryFromReader(message: GeoIpLocation, reader: jspb.BinaryReader): GeoIpLocation;
}

export namespace GeoIpLocation {
    export type AsObject = {
        ipv4?: string,
        ipv6?: string,
        country: string,
        city?: string,
        latitude: number,
        longitude: number,
        mullvadExitIp: boolean,
        hostname?: string,
        entryHostname?: string,
        obfuscatorHostname?: string,
    }
}

export class TunnelMetadata extends jspb.Message { 
    getTunnelInterface(): string;
    setTunnelInterface(value: string): TunnelMetadata;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TunnelMetadata.AsObject;
    static toObject(includeInstance: boolean, msg: TunnelMetadata): TunnelMetadata.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TunnelMetadata, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TunnelMetadata;
    static deserializeBinaryFromReader(message: TunnelMetadata, reader: jspb.BinaryReader): TunnelMetadata;
}

export namespace TunnelMetadata {
    export type AsObject = {
        tunnelInterface: string,
    }
}

export class LocationConstraint extends jspb.Message { 

    hasCustomList(): boolean;
    clearCustomList(): void;
    getCustomList(): string;
    setCustomList(value: string): LocationConstraint;

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): GeographicLocationConstraint | undefined;
    setLocation(value?: GeographicLocationConstraint): LocationConstraint;

    getTypeCase(): LocationConstraint.TypeCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LocationConstraint.AsObject;
    static toObject(includeInstance: boolean, msg: LocationConstraint): LocationConstraint.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LocationConstraint, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LocationConstraint;
    static deserializeBinaryFromReader(message: LocationConstraint, reader: jspb.BinaryReader): LocationConstraint;
}

export namespace LocationConstraint {
    export type AsObject = {
        customList: string,
        location?: GeographicLocationConstraint.AsObject,
    }

    export enum TypeCase {
        TYPE_NOT_SET = 0,
        CUSTOM_LIST = 1,
        LOCATION = 2,
    }

}

export class GeographicLocationConstraint extends jspb.Message { 
    getCountry(): string;
    setCountry(value: string): GeographicLocationConstraint;

    hasCity(): boolean;
    clearCity(): void;
    getCity(): string | undefined;
    setCity(value: string): GeographicLocationConstraint;

    hasHostname(): boolean;
    clearHostname(): void;
    getHostname(): string | undefined;
    setHostname(value: string): GeographicLocationConstraint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): GeographicLocationConstraint.AsObject;
    static toObject(includeInstance: boolean, msg: GeographicLocationConstraint): GeographicLocationConstraint.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: GeographicLocationConstraint, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): GeographicLocationConstraint;
    static deserializeBinaryFromReader(message: GeographicLocationConstraint, reader: jspb.BinaryReader): GeographicLocationConstraint;
}

export namespace GeographicLocationConstraint {
    export type AsObject = {
        country: string,
        city?: string,
        hostname?: string,
    }
}

export class ObfuscationSettings extends jspb.Message { 
    getSelectedObfuscation(): ObfuscationSettings.SelectedObfuscation;
    setSelectedObfuscation(value: ObfuscationSettings.SelectedObfuscation): ObfuscationSettings;

    hasUdp2tcp(): boolean;
    clearUdp2tcp(): void;
    getUdp2tcp(): ObfuscationSettings.Udp2TcpObfuscation | undefined;
    setUdp2tcp(value?: ObfuscationSettings.Udp2TcpObfuscation): ObfuscationSettings;

    hasShadowsocks(): boolean;
    clearShadowsocks(): void;
    getShadowsocks(): ObfuscationSettings.Shadowsocks | undefined;
    setShadowsocks(value?: ObfuscationSettings.Shadowsocks): ObfuscationSettings;

    hasWireguardPort(): boolean;
    clearWireguardPort(): void;
    getWireguardPort(): ObfuscationSettings.WireguardPort | undefined;
    setWireguardPort(value?: ObfuscationSettings.WireguardPort): ObfuscationSettings;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ObfuscationSettings.AsObject;
    static toObject(includeInstance: boolean, msg: ObfuscationSettings): ObfuscationSettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ObfuscationSettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ObfuscationSettings;
    static deserializeBinaryFromReader(message: ObfuscationSettings, reader: jspb.BinaryReader): ObfuscationSettings;
}

export namespace ObfuscationSettings {
    export type AsObject = {
        selectedObfuscation: ObfuscationSettings.SelectedObfuscation,
        udp2tcp?: ObfuscationSettings.Udp2TcpObfuscation.AsObject,
        shadowsocks?: ObfuscationSettings.Shadowsocks.AsObject,
        wireguardPort?: ObfuscationSettings.WireguardPort.AsObject,
    }


    export class Udp2TcpObfuscation extends jspb.Message { 

        hasPort(): boolean;
        clearPort(): void;
        getPort(): number | undefined;
        setPort(value: number): Udp2TcpObfuscation;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Udp2TcpObfuscation.AsObject;
        static toObject(includeInstance: boolean, msg: Udp2TcpObfuscation): Udp2TcpObfuscation.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Udp2TcpObfuscation, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Udp2TcpObfuscation;
        static deserializeBinaryFromReader(message: Udp2TcpObfuscation, reader: jspb.BinaryReader): Udp2TcpObfuscation;
    }

    export namespace Udp2TcpObfuscation {
        export type AsObject = {
            port?: number,
        }
    }

    export class Shadowsocks extends jspb.Message { 

        hasPort(): boolean;
        clearPort(): void;
        getPort(): number | undefined;
        setPort(value: number): Shadowsocks;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Shadowsocks.AsObject;
        static toObject(includeInstance: boolean, msg: Shadowsocks): Shadowsocks.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Shadowsocks, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Shadowsocks;
        static deserializeBinaryFromReader(message: Shadowsocks, reader: jspb.BinaryReader): Shadowsocks;
    }

    export namespace Shadowsocks {
        export type AsObject = {
            port?: number,
        }
    }

    export class WireguardPort extends jspb.Message { 

        hasPort(): boolean;
        clearPort(): void;
        getPort(): number | undefined;
        setPort(value: number): WireguardPort;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): WireguardPort.AsObject;
        static toObject(includeInstance: boolean, msg: WireguardPort): WireguardPort.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: WireguardPort, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): WireguardPort;
        static deserializeBinaryFromReader(message: WireguardPort, reader: jspb.BinaryReader): WireguardPort;
    }

    export namespace WireguardPort {
        export type AsObject = {
            port?: number,
        }
    }


    export enum SelectedObfuscation {
    AUTO = 0,
    OFF = 1,
    WIREGUARD_PORT = 2,
    UDP2TCP = 3,
    SHADOWSOCKS = 4,
    QUIC = 5,
    LWO = 6,
    }

}

export class CustomList extends jspb.Message { 
    getId(): string;
    setId(value: string): CustomList;
    getName(): string;
    setName(value: string): CustomList;
    clearLocationsList(): void;
    getLocationsList(): Array<GeographicLocationConstraint>;
    setLocationsList(value: Array<GeographicLocationConstraint>): CustomList;
    addLocations(value?: GeographicLocationConstraint, index?: number): GeographicLocationConstraint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CustomList.AsObject;
    static toObject(includeInstance: boolean, msg: CustomList): CustomList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CustomList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CustomList;
    static deserializeBinaryFromReader(message: CustomList, reader: jspb.BinaryReader): CustomList;
}

export namespace CustomList {
    export type AsObject = {
        id: string,
        name: string,
        locationsList: Array<GeographicLocationConstraint.AsObject>,
    }
}

export class NewCustomList extends jspb.Message { 
    getName(): string;
    setName(value: string): NewCustomList;
    clearLocationsList(): void;
    getLocationsList(): Array<GeographicLocationConstraint>;
    setLocationsList(value: Array<GeographicLocationConstraint>): NewCustomList;
    addLocations(value?: GeographicLocationConstraint, index?: number): GeographicLocationConstraint;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): NewCustomList.AsObject;
    static toObject(includeInstance: boolean, msg: NewCustomList): NewCustomList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: NewCustomList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): NewCustomList;
    static deserializeBinaryFromReader(message: NewCustomList, reader: jspb.BinaryReader): NewCustomList;
}

export namespace NewCustomList {
    export type AsObject = {
        name: string,
        locationsList: Array<GeographicLocationConstraint.AsObject>,
    }
}

export class CustomListSettings extends jspb.Message { 
    clearCustomListsList(): void;
    getCustomListsList(): Array<CustomList>;
    setCustomListsList(value: Array<CustomList>): CustomListSettings;
    addCustomLists(value?: CustomList, index?: number): CustomList;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CustomListSettings.AsObject;
    static toObject(includeInstance: boolean, msg: CustomListSettings): CustomListSettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CustomListSettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CustomListSettings;
    static deserializeBinaryFromReader(message: CustomListSettings, reader: jspb.BinaryReader): CustomListSettings;
}

export namespace CustomListSettings {
    export type AsObject = {
        customListsList: Array<CustomList.AsObject>,
    }
}

export class Socks5Local extends jspb.Message { 
    getRemoteIp(): string;
    setRemoteIp(value: string): Socks5Local;
    getRemotePort(): number;
    setRemotePort(value: number): Socks5Local;
    getRemoteTransportProtocol(): TransportProtocol;
    setRemoteTransportProtocol(value: TransportProtocol): Socks5Local;
    getLocalPort(): number;
    setLocalPort(value: number): Socks5Local;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Socks5Local.AsObject;
    static toObject(includeInstance: boolean, msg: Socks5Local): Socks5Local.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Socks5Local, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Socks5Local;
    static deserializeBinaryFromReader(message: Socks5Local, reader: jspb.BinaryReader): Socks5Local;
}

export namespace Socks5Local {
    export type AsObject = {
        remoteIp: string,
        remotePort: number,
        remoteTransportProtocol: TransportProtocol,
        localPort: number,
    }
}

export class SocksAuth extends jspb.Message { 
    getUsername(): string;
    setUsername(value: string): SocksAuth;
    getPassword(): string;
    setPassword(value: string): SocksAuth;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SocksAuth.AsObject;
    static toObject(includeInstance: boolean, msg: SocksAuth): SocksAuth.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SocksAuth, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SocksAuth;
    static deserializeBinaryFromReader(message: SocksAuth, reader: jspb.BinaryReader): SocksAuth;
}

export namespace SocksAuth {
    export type AsObject = {
        username: string,
        password: string,
    }
}

export class Socks5Remote extends jspb.Message { 
    getIp(): string;
    setIp(value: string): Socks5Remote;
    getPort(): number;
    setPort(value: number): Socks5Remote;

    hasAuth(): boolean;
    clearAuth(): void;
    getAuth(): SocksAuth | undefined;
    setAuth(value?: SocksAuth): Socks5Remote;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Socks5Remote.AsObject;
    static toObject(includeInstance: boolean, msg: Socks5Remote): Socks5Remote.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Socks5Remote, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Socks5Remote;
    static deserializeBinaryFromReader(message: Socks5Remote, reader: jspb.BinaryReader): Socks5Remote;
}

export namespace Socks5Remote {
    export type AsObject = {
        ip: string,
        port: number,
        auth?: SocksAuth.AsObject,
    }
}

export class Shadowsocks extends jspb.Message { 
    getIp(): string;
    setIp(value: string): Shadowsocks;
    getPort(): number;
    setPort(value: number): Shadowsocks;
    getPassword(): string;
    setPassword(value: string): Shadowsocks;
    getCipher(): string;
    setCipher(value: string): Shadowsocks;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Shadowsocks.AsObject;
    static toObject(includeInstance: boolean, msg: Shadowsocks): Shadowsocks.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Shadowsocks, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Shadowsocks;
    static deserializeBinaryFromReader(message: Shadowsocks, reader: jspb.BinaryReader): Shadowsocks;
}

export namespace Shadowsocks {
    export type AsObject = {
        ip: string,
        port: number,
        password: string,
        cipher: string,
    }
}

export class CustomProxy extends jspb.Message { 

    hasSocks5local(): boolean;
    clearSocks5local(): void;
    getSocks5local(): Socks5Local | undefined;
    setSocks5local(value?: Socks5Local): CustomProxy;

    hasSocks5remote(): boolean;
    clearSocks5remote(): void;
    getSocks5remote(): Socks5Remote | undefined;
    setSocks5remote(value?: Socks5Remote): CustomProxy;

    hasShadowsocks(): boolean;
    clearShadowsocks(): void;
    getShadowsocks(): Shadowsocks | undefined;
    setShadowsocks(value?: Shadowsocks): CustomProxy;

    getProxyMethodCase(): CustomProxy.ProxyMethodCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CustomProxy.AsObject;
    static toObject(includeInstance: boolean, msg: CustomProxy): CustomProxy.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CustomProxy, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CustomProxy;
    static deserializeBinaryFromReader(message: CustomProxy, reader: jspb.BinaryReader): CustomProxy;
}

export namespace CustomProxy {
    export type AsObject = {
        socks5local?: Socks5Local.AsObject,
        socks5remote?: Socks5Remote.AsObject,
        shadowsocks?: Shadowsocks.AsObject,
    }

    export enum ProxyMethodCase {
        PROXY_METHOD_NOT_SET = 0,
        SOCKS5LOCAL = 1,
        SOCKS5REMOTE = 2,
        SHADOWSOCKS = 3,
    }

}

export class AccessMethod extends jspb.Message { 

    hasDirect(): boolean;
    clearDirect(): void;
    getDirect(): AccessMethod.Direct | undefined;
    setDirect(value?: AccessMethod.Direct): AccessMethod;

    hasBridges(): boolean;
    clearBridges(): void;
    getBridges(): AccessMethod.Bridges | undefined;
    setBridges(value?: AccessMethod.Bridges): AccessMethod;

    hasEncryptedDnsProxy(): boolean;
    clearEncryptedDnsProxy(): void;
    getEncryptedDnsProxy(): AccessMethod.EncryptedDnsProxy | undefined;
    setEncryptedDnsProxy(value?: AccessMethod.EncryptedDnsProxy): AccessMethod;

    hasCustom(): boolean;
    clearCustom(): void;
    getCustom(): CustomProxy | undefined;
    setCustom(value?: CustomProxy): AccessMethod;

    getAccessMethodCase(): AccessMethod.AccessMethodCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccessMethod.AsObject;
    static toObject(includeInstance: boolean, msg: AccessMethod): AccessMethod.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccessMethod, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccessMethod;
    static deserializeBinaryFromReader(message: AccessMethod, reader: jspb.BinaryReader): AccessMethod;
}

export namespace AccessMethod {
    export type AsObject = {
        direct?: AccessMethod.Direct.AsObject,
        bridges?: AccessMethod.Bridges.AsObject,
        encryptedDnsProxy?: AccessMethod.EncryptedDnsProxy.AsObject,
        custom?: CustomProxy.AsObject,
    }


    export class Direct extends jspb.Message { 

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Direct.AsObject;
        static toObject(includeInstance: boolean, msg: Direct): Direct.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Direct, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Direct;
        static deserializeBinaryFromReader(message: Direct, reader: jspb.BinaryReader): Direct;
    }

    export namespace Direct {
        export type AsObject = {
        }
    }

    export class Bridges extends jspb.Message { 

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Bridges.AsObject;
        static toObject(includeInstance: boolean, msg: Bridges): Bridges.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Bridges, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Bridges;
        static deserializeBinaryFromReader(message: Bridges, reader: jspb.BinaryReader): Bridges;
    }

    export namespace Bridges {
        export type AsObject = {
        }
    }

    export class EncryptedDnsProxy extends jspb.Message { 

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): EncryptedDnsProxy.AsObject;
        static toObject(includeInstance: boolean, msg: EncryptedDnsProxy): EncryptedDnsProxy.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: EncryptedDnsProxy, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): EncryptedDnsProxy;
        static deserializeBinaryFromReader(message: EncryptedDnsProxy, reader: jspb.BinaryReader): EncryptedDnsProxy;
    }

    export namespace EncryptedDnsProxy {
        export type AsObject = {
        }
    }


    export enum AccessMethodCase {
        ACCESS_METHOD_NOT_SET = 0,
        DIRECT = 1,
        BRIDGES = 2,
        ENCRYPTED_DNS_PROXY = 3,
        CUSTOM = 4,
    }

}

export class AccessMethodSetting extends jspb.Message { 

    hasId(): boolean;
    clearId(): void;
    getId(): UUID | undefined;
    setId(value?: UUID): AccessMethodSetting;
    getName(): string;
    setName(value: string): AccessMethodSetting;
    getEnabled(): boolean;
    setEnabled(value: boolean): AccessMethodSetting;

    hasAccessMethod(): boolean;
    clearAccessMethod(): void;
    getAccessMethod(): AccessMethod | undefined;
    setAccessMethod(value?: AccessMethod): AccessMethodSetting;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccessMethodSetting.AsObject;
    static toObject(includeInstance: boolean, msg: AccessMethodSetting): AccessMethodSetting.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccessMethodSetting, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccessMethodSetting;
    static deserializeBinaryFromReader(message: AccessMethodSetting, reader: jspb.BinaryReader): AccessMethodSetting;
}

export namespace AccessMethodSetting {
    export type AsObject = {
        id?: UUID.AsObject,
        name: string,
        enabled: boolean,
        accessMethod?: AccessMethod.AsObject,
    }
}

export class NewAccessMethodSetting extends jspb.Message { 
    getName(): string;
    setName(value: string): NewAccessMethodSetting;
    getEnabled(): boolean;
    setEnabled(value: boolean): NewAccessMethodSetting;

    hasAccessMethod(): boolean;
    clearAccessMethod(): void;
    getAccessMethod(): AccessMethod | undefined;
    setAccessMethod(value?: AccessMethod): NewAccessMethodSetting;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): NewAccessMethodSetting.AsObject;
    static toObject(includeInstance: boolean, msg: NewAccessMethodSetting): NewAccessMethodSetting.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: NewAccessMethodSetting, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): NewAccessMethodSetting;
    static deserializeBinaryFromReader(message: NewAccessMethodSetting, reader: jspb.BinaryReader): NewAccessMethodSetting;
}

export namespace NewAccessMethodSetting {
    export type AsObject = {
        name: string,
        enabled: boolean,
        accessMethod?: AccessMethod.AsObject,
    }
}

export class ApiAccessMethodSettings extends jspb.Message { 

    hasDirect(): boolean;
    clearDirect(): void;
    getDirect(): AccessMethodSetting | undefined;
    setDirect(value?: AccessMethodSetting): ApiAccessMethodSettings;

    hasMullvadBridges(): boolean;
    clearMullvadBridges(): void;
    getMullvadBridges(): AccessMethodSetting | undefined;
    setMullvadBridges(value?: AccessMethodSetting): ApiAccessMethodSettings;

    hasEncryptedDnsProxy(): boolean;
    clearEncryptedDnsProxy(): void;
    getEncryptedDnsProxy(): AccessMethodSetting | undefined;
    setEncryptedDnsProxy(value?: AccessMethodSetting): ApiAccessMethodSettings;
    clearCustomList(): void;
    getCustomList(): Array<AccessMethodSetting>;
    setCustomList(value: Array<AccessMethodSetting>): ApiAccessMethodSettings;
    addCustom(value?: AccessMethodSetting, index?: number): AccessMethodSetting;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ApiAccessMethodSettings.AsObject;
    static toObject(includeInstance: boolean, msg: ApiAccessMethodSettings): ApiAccessMethodSettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ApiAccessMethodSettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ApiAccessMethodSettings;
    static deserializeBinaryFromReader(message: ApiAccessMethodSettings, reader: jspb.BinaryReader): ApiAccessMethodSettings;
}

export namespace ApiAccessMethodSettings {
    export type AsObject = {
        direct?: AccessMethodSetting.AsObject,
        mullvadBridges?: AccessMethodSetting.AsObject,
        encryptedDnsProxy?: AccessMethodSetting.AsObject,
        customList: Array<AccessMethodSetting.AsObject>,
    }
}

export class Settings extends jspb.Message { 

    hasRelaySettings(): boolean;
    clearRelaySettings(): void;
    getRelaySettings(): RelaySettings | undefined;
    setRelaySettings(value?: RelaySettings): Settings;
    getAllowLan(): boolean;
    setAllowLan(value: boolean): Settings;
    getLockdownMode(): boolean;
    setLockdownMode(value: boolean): Settings;
    getAutoConnect(): boolean;
    setAutoConnect(value: boolean): Settings;

    hasTunnelOptions(): boolean;
    clearTunnelOptions(): void;
    getTunnelOptions(): TunnelOptions | undefined;
    setTunnelOptions(value?: TunnelOptions): Settings;
    getShowBetaReleases(): boolean;
    setShowBetaReleases(value: boolean): Settings;

    hasSplitTunnel(): boolean;
    clearSplitTunnel(): void;
    getSplitTunnel(): SplitTunnelSettings | undefined;
    setSplitTunnel(value?: SplitTunnelSettings): Settings;

    hasObfuscationSettings(): boolean;
    clearObfuscationSettings(): void;
    getObfuscationSettings(): ObfuscationSettings | undefined;
    setObfuscationSettings(value?: ObfuscationSettings): Settings;

    hasCustomLists(): boolean;
    clearCustomLists(): void;
    getCustomLists(): CustomListSettings | undefined;
    setCustomLists(value?: CustomListSettings): Settings;

    hasApiAccessMethods(): boolean;
    clearApiAccessMethods(): void;
    getApiAccessMethods(): ApiAccessMethodSettings | undefined;
    setApiAccessMethods(value?: ApiAccessMethodSettings): Settings;
    clearRelayOverridesList(): void;
    getRelayOverridesList(): Array<RelayOverride>;
    setRelayOverridesList(value: Array<RelayOverride>): Settings;
    addRelayOverrides(value?: RelayOverride, index?: number): RelayOverride;

    hasRecents(): boolean;
    clearRecents(): void;
    getRecents(): Recents | undefined;
    setRecents(value?: Recents): Settings;
    getUpdateDefaultLocation(): boolean;
    setUpdateDefaultLocation(value: boolean): Settings;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Settings.AsObject;
    static toObject(includeInstance: boolean, msg: Settings): Settings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Settings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Settings;
    static deserializeBinaryFromReader(message: Settings, reader: jspb.BinaryReader): Settings;
}

export namespace Settings {
    export type AsObject = {
        relaySettings?: RelaySettings.AsObject,
        allowLan: boolean,
        lockdownMode: boolean,
        autoConnect: boolean,
        tunnelOptions?: TunnelOptions.AsObject,
        showBetaReleases: boolean,
        splitTunnel?: SplitTunnelSettings.AsObject,
        obfuscationSettings?: ObfuscationSettings.AsObject,
        customLists?: CustomListSettings.AsObject,
        apiAccessMethods?: ApiAccessMethodSettings.AsObject,
        relayOverridesList: Array<RelayOverride.AsObject>,
        recents?: Recents.AsObject,
        updateDefaultLocation: boolean,
    }
}

export class RelayOverride extends jspb.Message { 
    getHostname(): string;
    setHostname(value: string): RelayOverride;

    hasIpv4AddrIn(): boolean;
    clearIpv4AddrIn(): void;
    getIpv4AddrIn(): string | undefined;
    setIpv4AddrIn(value: string): RelayOverride;

    hasIpv6AddrIn(): boolean;
    clearIpv6AddrIn(): void;
    getIpv6AddrIn(): string | undefined;
    setIpv6AddrIn(value: string): RelayOverride;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelayOverride.AsObject;
    static toObject(includeInstance: boolean, msg: RelayOverride): RelayOverride.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelayOverride, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelayOverride;
    static deserializeBinaryFromReader(message: RelayOverride, reader: jspb.BinaryReader): RelayOverride;
}

export namespace RelayOverride {
    export type AsObject = {
        hostname: string,
        ipv4AddrIn?: string,
        ipv6AddrIn?: string,
    }
}

export class Recents extends jspb.Message { 
    clearRecentsList(): void;
    getRecentsList(): Array<Recent>;
    setRecentsList(value: Array<Recent>): Recents;
    addRecents(value?: Recent, index?: number): Recent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Recents.AsObject;
    static toObject(includeInstance: boolean, msg: Recents): Recents.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Recents, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Recents;
    static deserializeBinaryFromReader(message: Recents, reader: jspb.BinaryReader): Recents;
}

export namespace Recents {
    export type AsObject = {
        recentsList: Array<Recent.AsObject>,
    }
}

export class MultihopRecent extends jspb.Message { 

    hasEntry(): boolean;
    clearEntry(): void;
    getEntry(): LocationConstraint | undefined;
    setEntry(value?: LocationConstraint): MultihopRecent;

    hasExit(): boolean;
    clearExit(): void;
    getExit(): LocationConstraint | undefined;
    setExit(value?: LocationConstraint): MultihopRecent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): MultihopRecent.AsObject;
    static toObject(includeInstance: boolean, msg: MultihopRecent): MultihopRecent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: MultihopRecent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): MultihopRecent;
    static deserializeBinaryFromReader(message: MultihopRecent, reader: jspb.BinaryReader): MultihopRecent;
}

export namespace MultihopRecent {
    export type AsObject = {
        entry?: LocationConstraint.AsObject,
        exit?: LocationConstraint.AsObject,
    }
}

export class Recent extends jspb.Message { 

    hasSinglehop(): boolean;
    clearSinglehop(): void;
    getSinglehop(): LocationConstraint | undefined;
    setSinglehop(value?: LocationConstraint): Recent;

    hasMultihop(): boolean;
    clearMultihop(): void;
    getMultihop(): MultihopRecent | undefined;
    setMultihop(value?: MultihopRecent): Recent;

    getTypeCase(): Recent.TypeCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Recent.AsObject;
    static toObject(includeInstance: boolean, msg: Recent): Recent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Recent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Recent;
    static deserializeBinaryFromReader(message: Recent, reader: jspb.BinaryReader): Recent;
}

export namespace Recent {
    export type AsObject = {
        singlehop?: LocationConstraint.AsObject,
        multihop?: MultihopRecent.AsObject,
    }

    export enum TypeCase {
        TYPE_NOT_SET = 0,
        SINGLEHOP = 1,
        MULTIHOP = 2,
    }

}

export class SplitTunnelSettings extends jspb.Message { 
    getEnableExclusions(): boolean;
    setEnableExclusions(value: boolean): SplitTunnelSettings;
    clearAppsList(): void;
    getAppsList(): Array<string>;
    setAppsList(value: Array<string>): SplitTunnelSettings;
    addApps(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SplitTunnelSettings.AsObject;
    static toObject(includeInstance: boolean, msg: SplitTunnelSettings): SplitTunnelSettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SplitTunnelSettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SplitTunnelSettings;
    static deserializeBinaryFromReader(message: SplitTunnelSettings, reader: jspb.BinaryReader): SplitTunnelSettings;
}

export namespace SplitTunnelSettings {
    export type AsObject = {
        enableExclusions: boolean,
        appsList: Array<string>,
    }
}

export class RelaySettings extends jspb.Message { 

    hasCustom(): boolean;
    clearCustom(): void;
    getCustom(): CustomRelaySettings | undefined;
    setCustom(value?: CustomRelaySettings): RelaySettings;

    hasNormal(): boolean;
    clearNormal(): void;
    getNormal(): NormalRelaySettings | undefined;
    setNormal(value?: NormalRelaySettings): RelaySettings;

    getEndpointCase(): RelaySettings.EndpointCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelaySettings.AsObject;
    static toObject(includeInstance: boolean, msg: RelaySettings): RelaySettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelaySettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelaySettings;
    static deserializeBinaryFromReader(message: RelaySettings, reader: jspb.BinaryReader): RelaySettings;
}

export namespace RelaySettings {
    export type AsObject = {
        custom?: CustomRelaySettings.AsObject,
        normal?: NormalRelaySettings.AsObject,
    }

    export enum EndpointCase {
        ENDPOINT_NOT_SET = 0,
        CUSTOM = 1,
        NORMAL = 2,
    }

}

export class NormalRelaySettings extends jspb.Message { 

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): LocationConstraint | undefined;
    setLocation(value?: LocationConstraint): NormalRelaySettings;
    clearProvidersList(): void;
    getProvidersList(): Array<string>;
    setProvidersList(value: Array<string>): NormalRelaySettings;
    addProviders(value: string, index?: number): string;

    hasWireguardConstraints(): boolean;
    clearWireguardConstraints(): void;
    getWireguardConstraints(): WireguardConstraints | undefined;
    setWireguardConstraints(value?: WireguardConstraints): NormalRelaySettings;
    getOwnership(): Ownership;
    setOwnership(value: Ownership): NormalRelaySettings;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): NormalRelaySettings.AsObject;
    static toObject(includeInstance: boolean, msg: NormalRelaySettings): NormalRelaySettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: NormalRelaySettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): NormalRelaySettings;
    static deserializeBinaryFromReader(message: NormalRelaySettings, reader: jspb.BinaryReader): NormalRelaySettings;
}

export namespace NormalRelaySettings {
    export type AsObject = {
        location?: LocationConstraint.AsObject,
        providersList: Array<string>,
        wireguardConstraints?: WireguardConstraints.AsObject,
        ownership: Ownership,
    }
}

export class TransportPort extends jspb.Message { 
    getProtocol(): TransportProtocol;
    setProtocol(value: TransportProtocol): TransportPort;

    hasPort(): boolean;
    clearPort(): void;
    getPort(): number | undefined;
    setPort(value: number): TransportPort;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TransportPort.AsObject;
    static toObject(includeInstance: boolean, msg: TransportPort): TransportPort.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TransportPort, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TransportPort;
    static deserializeBinaryFromReader(message: TransportPort, reader: jspb.BinaryReader): TransportPort;
}

export namespace TransportPort {
    export type AsObject = {
        protocol: TransportProtocol,
        port?: number,
    }
}

export class WireguardConstraints extends jspb.Message { 

    hasIpVersion(): boolean;
    clearIpVersion(): void;
    getIpVersion(): IpVersion | undefined;
    setIpVersion(value: IpVersion): WireguardConstraints;
    clearAllowedIpsList(): void;
    getAllowedIpsList(): Array<string>;
    setAllowedIpsList(value: Array<string>): WireguardConstraints;
    addAllowedIps(value: string, index?: number): string;
    getUseMultihop(): boolean;
    setUseMultihop(value: boolean): WireguardConstraints;

    hasEntryLocation(): boolean;
    clearEntryLocation(): void;
    getEntryLocation(): LocationConstraint | undefined;
    setEntryLocation(value?: LocationConstraint): WireguardConstraints;
    clearEntryProvidersList(): void;
    getEntryProvidersList(): Array<string>;
    setEntryProvidersList(value: Array<string>): WireguardConstraints;
    addEntryProviders(value: string, index?: number): string;
    getEntryOwnership(): Ownership;
    setEntryOwnership(value: Ownership): WireguardConstraints;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): WireguardConstraints.AsObject;
    static toObject(includeInstance: boolean, msg: WireguardConstraints): WireguardConstraints.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: WireguardConstraints, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): WireguardConstraints;
    static deserializeBinaryFromReader(message: WireguardConstraints, reader: jspb.BinaryReader): WireguardConstraints;
}

export namespace WireguardConstraints {
    export type AsObject = {
        ipVersion?: IpVersion,
        allowedIpsList: Array<string>,
        useMultihop: boolean,
        entryLocation?: LocationConstraint.AsObject,
        entryProvidersList: Array<string>,
        entryOwnership: Ownership,
    }
}

export class CustomRelaySettings extends jspb.Message { 
    getHost(): string;
    setHost(value: string): CustomRelaySettings;

    hasConfig(): boolean;
    clearConfig(): void;
    getConfig(): WireguardConfig | undefined;
    setConfig(value?: WireguardConfig): CustomRelaySettings;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CustomRelaySettings.AsObject;
    static toObject(includeInstance: boolean, msg: CustomRelaySettings): CustomRelaySettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CustomRelaySettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CustomRelaySettings;
    static deserializeBinaryFromReader(message: CustomRelaySettings, reader: jspb.BinaryReader): CustomRelaySettings;
}

export namespace CustomRelaySettings {
    export type AsObject = {
        host: string,
        config?: WireguardConfig.AsObject,
    }
}

export class WireguardConfig extends jspb.Message { 

    hasTunnel(): boolean;
    clearTunnel(): void;
    getTunnel(): WireguardConfig.TunnelConfig | undefined;
    setTunnel(value?: WireguardConfig.TunnelConfig): WireguardConfig;

    hasPeer(): boolean;
    clearPeer(): void;
    getPeer(): WireguardConfig.PeerConfig | undefined;
    setPeer(value?: WireguardConfig.PeerConfig): WireguardConfig;
    getIpv4Gateway(): string;
    setIpv4Gateway(value: string): WireguardConfig;

    hasIpv6Gateway(): boolean;
    clearIpv6Gateway(): void;
    getIpv6Gateway(): string | undefined;
    setIpv6Gateway(value: string): WireguardConfig;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): WireguardConfig.AsObject;
    static toObject(includeInstance: boolean, msg: WireguardConfig): WireguardConfig.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: WireguardConfig, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): WireguardConfig;
    static deserializeBinaryFromReader(message: WireguardConfig, reader: jspb.BinaryReader): WireguardConfig;
}

export namespace WireguardConfig {
    export type AsObject = {
        tunnel?: WireguardConfig.TunnelConfig.AsObject,
        peer?: WireguardConfig.PeerConfig.AsObject,
        ipv4Gateway: string,
        ipv6Gateway?: string,
    }


    export class TunnelConfig extends jspb.Message { 
        getPrivateKey(): Uint8Array | string;
        getPrivateKey_asU8(): Uint8Array;
        getPrivateKey_asB64(): string;
        setPrivateKey(value: Uint8Array | string): TunnelConfig;
        clearAddressesList(): void;
        getAddressesList(): Array<string>;
        setAddressesList(value: Array<string>): TunnelConfig;
        addAddresses(value: string, index?: number): string;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): TunnelConfig.AsObject;
        static toObject(includeInstance: boolean, msg: TunnelConfig): TunnelConfig.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: TunnelConfig, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): TunnelConfig;
        static deserializeBinaryFromReader(message: TunnelConfig, reader: jspb.BinaryReader): TunnelConfig;
    }

    export namespace TunnelConfig {
        export type AsObject = {
            privateKey: Uint8Array | string,
            addressesList: Array<string>,
        }
    }

    export class PeerConfig extends jspb.Message { 
        getPublicKey(): Uint8Array | string;
        getPublicKey_asU8(): Uint8Array;
        getPublicKey_asB64(): string;
        setPublicKey(value: Uint8Array | string): PeerConfig;
        clearAllowedIpsList(): void;
        getAllowedIpsList(): Array<string>;
        setAllowedIpsList(value: Array<string>): PeerConfig;
        addAllowedIps(value: string, index?: number): string;
        getEndpoint(): string;
        setEndpoint(value: string): PeerConfig;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): PeerConfig.AsObject;
        static toObject(includeInstance: boolean, msg: PeerConfig): PeerConfig.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: PeerConfig, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): PeerConfig;
        static deserializeBinaryFromReader(message: PeerConfig, reader: jspb.BinaryReader): PeerConfig;
    }

    export namespace PeerConfig {
        export type AsObject = {
            publicKey: Uint8Array | string,
            allowedIpsList: Array<string>,
            endpoint: string,
        }
    }

}

export class QuantumResistantState extends jspb.Message { 
    getState(): QuantumResistantState.State;
    setState(value: QuantumResistantState.State): QuantumResistantState;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): QuantumResistantState.AsObject;
    static toObject(includeInstance: boolean, msg: QuantumResistantState): QuantumResistantState.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: QuantumResistantState, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): QuantumResistantState;
    static deserializeBinaryFromReader(message: QuantumResistantState, reader: jspb.BinaryReader): QuantumResistantState;
}

export namespace QuantumResistantState {
    export type AsObject = {
        state: QuantumResistantState.State,
    }

    export enum State {
    ON = 0,
    OFF = 1,
    }

}

export class DaitaSettings extends jspb.Message { 
    getEnabled(): boolean;
    setEnabled(value: boolean): DaitaSettings;
    getDirectOnly(): boolean;
    setDirectOnly(value: boolean): DaitaSettings;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DaitaSettings.AsObject;
    static toObject(includeInstance: boolean, msg: DaitaSettings): DaitaSettings.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DaitaSettings, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DaitaSettings;
    static deserializeBinaryFromReader(message: DaitaSettings, reader: jspb.BinaryReader): DaitaSettings;
}

export namespace DaitaSettings {
    export type AsObject = {
        enabled: boolean,
        directOnly: boolean,
    }
}

export class TunnelOptions extends jspb.Message { 

    hasMtu(): boolean;
    clearMtu(): void;
    getMtu(): number | undefined;
    setMtu(value: number): TunnelOptions;

    hasRotationInterval(): boolean;
    clearRotationInterval(): void;
    getRotationInterval(): google_protobuf_duration_pb.Duration | undefined;
    setRotationInterval(value?: google_protobuf_duration_pb.Duration): TunnelOptions;

    hasQuantumResistant(): boolean;
    clearQuantumResistant(): void;
    getQuantumResistant(): QuantumResistantState | undefined;
    setQuantumResistant(value?: QuantumResistantState): TunnelOptions;

    hasDaita(): boolean;
    clearDaita(): void;
    getDaita(): DaitaSettings | undefined;
    setDaita(value?: DaitaSettings): TunnelOptions;
    getEnableIpv6(): boolean;
    setEnableIpv6(value: boolean): TunnelOptions;

    hasDnsOptions(): boolean;
    clearDnsOptions(): void;
    getDnsOptions(): DnsOptions | undefined;
    setDnsOptions(value?: DnsOptions): TunnelOptions;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): TunnelOptions.AsObject;
    static toObject(includeInstance: boolean, msg: TunnelOptions): TunnelOptions.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: TunnelOptions, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): TunnelOptions;
    static deserializeBinaryFromReader(message: TunnelOptions, reader: jspb.BinaryReader): TunnelOptions;
}

export namespace TunnelOptions {
    export type AsObject = {
        mtu?: number,
        rotationInterval?: google_protobuf_duration_pb.Duration.AsObject,
        quantumResistant?: QuantumResistantState.AsObject,
        daita?: DaitaSettings.AsObject,
        enableIpv6: boolean,
        dnsOptions?: DnsOptions.AsObject,
    }
}

export class DefaultDnsOptions extends jspb.Message { 
    getBlockAds(): boolean;
    setBlockAds(value: boolean): DefaultDnsOptions;
    getBlockTrackers(): boolean;
    setBlockTrackers(value: boolean): DefaultDnsOptions;
    getBlockMalware(): boolean;
    setBlockMalware(value: boolean): DefaultDnsOptions;
    getBlockAdultContent(): boolean;
    setBlockAdultContent(value: boolean): DefaultDnsOptions;
    getBlockGambling(): boolean;
    setBlockGambling(value: boolean): DefaultDnsOptions;
    getBlockSocialMedia(): boolean;
    setBlockSocialMedia(value: boolean): DefaultDnsOptions;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DefaultDnsOptions.AsObject;
    static toObject(includeInstance: boolean, msg: DefaultDnsOptions): DefaultDnsOptions.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DefaultDnsOptions, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DefaultDnsOptions;
    static deserializeBinaryFromReader(message: DefaultDnsOptions, reader: jspb.BinaryReader): DefaultDnsOptions;
}

export namespace DefaultDnsOptions {
    export type AsObject = {
        blockAds: boolean,
        blockTrackers: boolean,
        blockMalware: boolean,
        blockAdultContent: boolean,
        blockGambling: boolean,
        blockSocialMedia: boolean,
    }
}

export class CustomDnsOptions extends jspb.Message { 
    clearAddressesList(): void;
    getAddressesList(): Array<string>;
    setAddressesList(value: Array<string>): CustomDnsOptions;
    addAddresses(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): CustomDnsOptions.AsObject;
    static toObject(includeInstance: boolean, msg: CustomDnsOptions): CustomDnsOptions.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: CustomDnsOptions, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): CustomDnsOptions;
    static deserializeBinaryFromReader(message: CustomDnsOptions, reader: jspb.BinaryReader): CustomDnsOptions;
}

export namespace CustomDnsOptions {
    export type AsObject = {
        addressesList: Array<string>,
    }
}

export class DnsOptions extends jspb.Message { 
    getState(): DnsOptions.DnsState;
    setState(value: DnsOptions.DnsState): DnsOptions;

    hasDefaultOptions(): boolean;
    clearDefaultOptions(): void;
    getDefaultOptions(): DefaultDnsOptions | undefined;
    setDefaultOptions(value?: DefaultDnsOptions): DnsOptions;

    hasCustomOptions(): boolean;
    clearCustomOptions(): void;
    getCustomOptions(): CustomDnsOptions | undefined;
    setCustomOptions(value?: CustomDnsOptions): DnsOptions;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DnsOptions.AsObject;
    static toObject(includeInstance: boolean, msg: DnsOptions): DnsOptions.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DnsOptions, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DnsOptions;
    static deserializeBinaryFromReader(message: DnsOptions, reader: jspb.BinaryReader): DnsOptions;
}

export namespace DnsOptions {
    export type AsObject = {
        state: DnsOptions.DnsState,
        defaultOptions?: DefaultDnsOptions.AsObject,
        customOptions?: CustomDnsOptions.AsObject,
    }

    export enum DnsState {
    DEFAULT = 0,
    CUSTOM = 1,
    }

}

export class PublicKey extends jspb.Message { 
    getKey(): Uint8Array | string;
    getKey_asU8(): Uint8Array;
    getKey_asB64(): string;
    setKey(value: Uint8Array | string): PublicKey;

    hasCreated(): boolean;
    clearCreated(): void;
    getCreated(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setCreated(value?: google_protobuf_timestamp_pb.Timestamp): PublicKey;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PublicKey.AsObject;
    static toObject(includeInstance: boolean, msg: PublicKey): PublicKey.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PublicKey, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PublicKey;
    static deserializeBinaryFromReader(message: PublicKey, reader: jspb.BinaryReader): PublicKey;
}

export namespace PublicKey {
    export type AsObject = {
        key: Uint8Array | string,
        created?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    }
}

export class ExcludedProcess extends jspb.Message { 
    getPid(): number;
    setPid(value: number): ExcludedProcess;
    getImage(): string;
    setImage(value: string): ExcludedProcess;
    getInherited(): boolean;
    setInherited(value: boolean): ExcludedProcess;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ExcludedProcess.AsObject;
    static toObject(includeInstance: boolean, msg: ExcludedProcess): ExcludedProcess.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ExcludedProcess, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ExcludedProcess;
    static deserializeBinaryFromReader(message: ExcludedProcess, reader: jspb.BinaryReader): ExcludedProcess;
}

export namespace ExcludedProcess {
    export type AsObject = {
        pid: number,
        image: string,
        inherited: boolean,
    }
}

export class ExcludedProcessList extends jspb.Message { 
    clearProcessesList(): void;
    getProcessesList(): Array<ExcludedProcess>;
    setProcessesList(value: Array<ExcludedProcess>): ExcludedProcessList;
    addProcesses(value?: ExcludedProcess, index?: number): ExcludedProcess;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ExcludedProcessList.AsObject;
    static toObject(includeInstance: boolean, msg: ExcludedProcessList): ExcludedProcessList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ExcludedProcessList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ExcludedProcessList;
    static deserializeBinaryFromReader(message: ExcludedProcessList, reader: jspb.BinaryReader): ExcludedProcessList;
}

export namespace ExcludedProcessList {
    export type AsObject = {
        processesList: Array<ExcludedProcess.AsObject>,
    }
}

export class SuggestedUpgrade extends jspb.Message { 
    getVersion(): string;
    setVersion(value: string): SuggestedUpgrade;
    getChangelog(): string;
    setChangelog(value: string): SuggestedUpgrade;

    hasVerifiedInstallerPath(): boolean;
    clearVerifiedInstallerPath(): void;
    getVerifiedInstallerPath(): string | undefined;
    setVerifiedInstallerPath(value: string): SuggestedUpgrade;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): SuggestedUpgrade.AsObject;
    static toObject(includeInstance: boolean, msg: SuggestedUpgrade): SuggestedUpgrade.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: SuggestedUpgrade, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): SuggestedUpgrade;
    static deserializeBinaryFromReader(message: SuggestedUpgrade, reader: jspb.BinaryReader): SuggestedUpgrade;
}

export namespace SuggestedUpgrade {
    export type AsObject = {
        version: string,
        changelog: string,
        verifiedInstallerPath?: string,
    }
}

export class AppVersionInfo extends jspb.Message { 
    getSupported(): boolean;
    setSupported(value: boolean): AppVersionInfo;

    hasSuggestedUpgrade(): boolean;
    clearSuggestedUpgrade(): void;
    getSuggestedUpgrade(): SuggestedUpgrade | undefined;
    setSuggestedUpgrade(value?: SuggestedUpgrade): AppVersionInfo;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AppVersionInfo.AsObject;
    static toObject(includeInstance: boolean, msg: AppVersionInfo): AppVersionInfo.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AppVersionInfo, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AppVersionInfo;
    static deserializeBinaryFromReader(message: AppVersionInfo, reader: jspb.BinaryReader): AppVersionInfo;
}

export namespace AppVersionInfo {
    export type AsObject = {
        supported: boolean,
        suggestedUpgrade?: SuggestedUpgrade.AsObject,
    }
}

export class RelayListCountry extends jspb.Message { 
    getName(): string;
    setName(value: string): RelayListCountry;
    getCode(): string;
    setCode(value: string): RelayListCountry;
    clearCitiesList(): void;
    getCitiesList(): Array<RelayListCity>;
    setCitiesList(value: Array<RelayListCity>): RelayListCountry;
    addCities(value?: RelayListCity, index?: number): RelayListCity;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelayListCountry.AsObject;
    static toObject(includeInstance: boolean, msg: RelayListCountry): RelayListCountry.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelayListCountry, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelayListCountry;
    static deserializeBinaryFromReader(message: RelayListCountry, reader: jspb.BinaryReader): RelayListCountry;
}

export namespace RelayListCountry {
    export type AsObject = {
        name: string,
        code: string,
        citiesList: Array<RelayListCity.AsObject>,
    }
}

export class RelayListCity extends jspb.Message { 
    getName(): string;
    setName(value: string): RelayListCity;
    getCode(): string;
    setCode(value: string): RelayListCity;
    getLatitude(): number;
    setLatitude(value: number): RelayListCity;
    getLongitude(): number;
    setLongitude(value: number): RelayListCity;
    clearRelaysList(): void;
    getRelaysList(): Array<Relay>;
    setRelaysList(value: Array<Relay>): RelayListCity;
    addRelays(value?: Relay, index?: number): Relay;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelayListCity.AsObject;
    static toObject(includeInstance: boolean, msg: RelayListCity): RelayListCity.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelayListCity, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelayListCity;
    static deserializeBinaryFromReader(message: RelayListCity, reader: jspb.BinaryReader): RelayListCity;
}

export namespace RelayListCity {
    export type AsObject = {
        name: string,
        code: string,
        latitude: number,
        longitude: number,
        relaysList: Array<Relay.AsObject>,
    }
}

export class Relay extends jspb.Message { 
    getHostname(): string;
    setHostname(value: string): Relay;
    getIpv4AddrIn(): string;
    setIpv4AddrIn(value: string): Relay;

    hasIpv6AddrIn(): boolean;
    clearIpv6AddrIn(): void;
    getIpv6AddrIn(): string | undefined;
    setIpv6AddrIn(value: string): Relay;
    getIncludeInCountry(): boolean;
    setIncludeInCountry(value: boolean): Relay;
    getActive(): boolean;
    setActive(value: boolean): Relay;
    getOwned(): boolean;
    setOwned(value: boolean): Relay;
    getProvider(): string;
    setProvider(value: string): Relay;
    getWeight(): number;
    setWeight(value: number): Relay;

    hasEndpointData(): boolean;
    clearEndpointData(): void;
    getEndpointData(): Relay.WireguardEndpoint | undefined;
    setEndpointData(value?: Relay.WireguardEndpoint): Relay;

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): Location | undefined;
    setLocation(value?: Location): Relay;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Relay.AsObject;
    static toObject(includeInstance: boolean, msg: Relay): Relay.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Relay, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Relay;
    static deserializeBinaryFromReader(message: Relay, reader: jspb.BinaryReader): Relay;
}

export namespace Relay {
    export type AsObject = {
        hostname: string,
        ipv4AddrIn: string,
        ipv6AddrIn?: string,
        includeInCountry: boolean,
        active: boolean,
        owned: boolean,
        provider: string,
        weight: number,
        endpointData?: Relay.WireguardEndpoint.AsObject,
        location?: Location.AsObject,
    }


    export class WireguardEndpoint extends jspb.Message { 
        getPublicKey(): Uint8Array | string;
        getPublicKey_asU8(): Uint8Array;
        getPublicKey_asB64(): string;
        setPublicKey(value: Uint8Array | string): WireguardEndpoint;
        getDaita(): boolean;
        setDaita(value: boolean): WireguardEndpoint;

        hasQuic(): boolean;
        clearQuic(): void;
        getQuic(): Relay.WireguardEndpoint.Quic | undefined;
        setQuic(value?: Relay.WireguardEndpoint.Quic): WireguardEndpoint;
        clearShadowsocksExtraAddrInList(): void;
        getShadowsocksExtraAddrInList(): Array<string>;
        setShadowsocksExtraAddrInList(value: Array<string>): WireguardEndpoint;
        addShadowsocksExtraAddrIn(value: string, index?: number): string;
        getLwo(): boolean;
        setLwo(value: boolean): WireguardEndpoint;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): WireguardEndpoint.AsObject;
        static toObject(includeInstance: boolean, msg: WireguardEndpoint): WireguardEndpoint.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: WireguardEndpoint, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): WireguardEndpoint;
        static deserializeBinaryFromReader(message: WireguardEndpoint, reader: jspb.BinaryReader): WireguardEndpoint;
    }

    export namespace WireguardEndpoint {
        export type AsObject = {
            publicKey: Uint8Array | string,
            daita: boolean,
            quic?: Relay.WireguardEndpoint.Quic.AsObject,
            shadowsocksExtraAddrInList: Array<string>,
            lwo: boolean,
        }


        export class Quic extends jspb.Message { 
            getDomain(): string;
            setDomain(value: string): Quic;
            getToken(): string;
            setToken(value: string): Quic;
            clearAddrInList(): void;
            getAddrInList(): Array<string>;
            setAddrInList(value: Array<string>): Quic;
            addAddrIn(value: string, index?: number): string;

            serializeBinary(): Uint8Array;
            toObject(includeInstance?: boolean): Quic.AsObject;
            static toObject(includeInstance: boolean, msg: Quic): Quic.AsObject;
            static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
            static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
            static serializeBinaryToWriter(message: Quic, writer: jspb.BinaryWriter): void;
            static deserializeBinary(bytes: Uint8Array): Quic;
            static deserializeBinaryFromReader(message: Quic, reader: jspb.BinaryReader): Quic;
        }

        export namespace Quic {
            export type AsObject = {
                domain: string,
                token: string,
                addrInList: Array<string>,
            }
        }

    }

}

export class Location extends jspb.Message { 
    getCountry(): string;
    setCountry(value: string): Location;
    getCountryCode(): string;
    setCountryCode(value: string): Location;
    getCity(): string;
    setCity(value: string): Location;
    getCityCode(): string;
    setCityCode(value: string): Location;
    getLatitude(): number;
    setLatitude(value: number): Location;
    getLongitude(): number;
    setLongitude(value: number): Location;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Location.AsObject;
    static toObject(includeInstance: boolean, msg: Location): Location.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Location, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Location;
    static deserializeBinaryFromReader(message: Location, reader: jspb.BinaryReader): Location;
}

export namespace Location {
    export type AsObject = {
        country: string,
        countryCode: string,
        city: string,
        cityCode: string,
        latitude: number,
        longitude: number,
    }
}

export class DaemonEvent extends jspb.Message { 

    hasTunnelState(): boolean;
    clearTunnelState(): void;
    getTunnelState(): TunnelState | undefined;
    setTunnelState(value?: TunnelState): DaemonEvent;

    hasSettings(): boolean;
    clearSettings(): void;
    getSettings(): Settings | undefined;
    setSettings(value?: Settings): DaemonEvent;

    hasRelayList(): boolean;
    clearRelayList(): void;
    getRelayList(): RelayList | undefined;
    setRelayList(value?: RelayList): DaemonEvent;

    hasVersionInfo(): boolean;
    clearVersionInfo(): void;
    getVersionInfo(): AppVersionInfo | undefined;
    setVersionInfo(value?: AppVersionInfo): DaemonEvent;

    hasDevice(): boolean;
    clearDevice(): void;
    getDevice(): DeviceEvent | undefined;
    setDevice(value?: DeviceEvent): DaemonEvent;

    hasRemoveDevice(): boolean;
    clearRemoveDevice(): void;
    getRemoveDevice(): RemoveDeviceEvent | undefined;
    setRemoveDevice(value?: RemoveDeviceEvent): DaemonEvent;

    hasNewAccessMethod(): boolean;
    clearNewAccessMethod(): void;
    getNewAccessMethod(): AccessMethodSetting | undefined;
    setNewAccessMethod(value?: AccessMethodSetting): DaemonEvent;

    hasLeakInfo(): boolean;
    clearLeakInfo(): void;
    getLeakInfo(): LeakInfo | undefined;
    setLeakInfo(value?: LeakInfo): DaemonEvent;

    getEventCase(): DaemonEvent.EventCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DaemonEvent.AsObject;
    static toObject(includeInstance: boolean, msg: DaemonEvent): DaemonEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DaemonEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DaemonEvent;
    static deserializeBinaryFromReader(message: DaemonEvent, reader: jspb.BinaryReader): DaemonEvent;
}

export namespace DaemonEvent {
    export type AsObject = {
        tunnelState?: TunnelState.AsObject,
        settings?: Settings.AsObject,
        relayList?: RelayList.AsObject,
        versionInfo?: AppVersionInfo.AsObject,
        device?: DeviceEvent.AsObject,
        removeDevice?: RemoveDeviceEvent.AsObject,
        newAccessMethod?: AccessMethodSetting.AsObject,
        leakInfo?: LeakInfo.AsObject,
    }

    export enum EventCase {
        EVENT_NOT_SET = 0,
        TUNNEL_STATE = 1,
        SETTINGS = 2,
        RELAY_LIST = 3,
        VERSION_INFO = 4,
        DEVICE = 5,
        REMOVE_DEVICE = 6,
        NEW_ACCESS_METHOD = 7,
        LEAK_INFO = 8,
    }

}

export class RelayList extends jspb.Message { 
    clearCountriesList(): void;
    getCountriesList(): Array<RelayListCountry>;
    setCountriesList(value: Array<RelayListCountry>): RelayList;
    addCountries(value?: RelayListCountry, index?: number): RelayListCountry;

    hasEndpointData(): boolean;
    clearEndpointData(): void;
    getEndpointData(): WireguardEndpointData | undefined;
    setEndpointData(value?: WireguardEndpointData): RelayList;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelayList.AsObject;
    static toObject(includeInstance: boolean, msg: RelayList): RelayList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelayList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelayList;
    static deserializeBinaryFromReader(message: RelayList, reader: jspb.BinaryReader): RelayList;
}

export namespace RelayList {
    export type AsObject = {
        countriesList: Array<RelayListCountry.AsObject>,
        endpointData?: WireguardEndpointData.AsObject,
    }
}

export class BridgeList extends jspb.Message { 
    clearBridgesList(): void;
    getBridgesList(): Array<Bridge>;
    setBridgesList(value: Array<Bridge>): BridgeList;
    addBridges(value?: Bridge, index?: number): Bridge;

    hasEndpointData(): boolean;
    clearEndpointData(): void;
    getEndpointData(): BridgeEndpointData | undefined;
    setEndpointData(value?: BridgeEndpointData): BridgeList;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): BridgeList.AsObject;
    static toObject(includeInstance: boolean, msg: BridgeList): BridgeList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: BridgeList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): BridgeList;
    static deserializeBinaryFromReader(message: BridgeList, reader: jspb.BinaryReader): BridgeList;
}

export namespace BridgeList {
    export type AsObject = {
        bridgesList: Array<Bridge.AsObject>,
        endpointData?: BridgeEndpointData.AsObject,
    }
}

export class Bridge extends jspb.Message { 
    getHostname(): string;
    setHostname(value: string): Bridge;
    getIpv4AddrIn(): string;
    setIpv4AddrIn(value: string): Bridge;

    hasIpv6AddrIn(): boolean;
    clearIpv6AddrIn(): void;
    getIpv6AddrIn(): string | undefined;
    setIpv6AddrIn(value: string): Bridge;
    getActive(): boolean;
    setActive(value: boolean): Bridge;
    getWeight(): number;
    setWeight(value: number): Bridge;

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): Location | undefined;
    setLocation(value?: Location): Bridge;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Bridge.AsObject;
    static toObject(includeInstance: boolean, msg: Bridge): Bridge.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Bridge, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Bridge;
    static deserializeBinaryFromReader(message: Bridge, reader: jspb.BinaryReader): Bridge;
}

export namespace Bridge {
    export type AsObject = {
        hostname: string,
        ipv4AddrIn: string,
        ipv6AddrIn?: string,
        active: boolean,
        weight: number,
        location?: Location.AsObject,
    }
}

export class BridgeEndpointData extends jspb.Message { 
    clearShadowsocksList(): void;
    getShadowsocksList(): Array<ShadowsocksEndpointData>;
    setShadowsocksList(value: Array<ShadowsocksEndpointData>): BridgeEndpointData;
    addShadowsocks(value?: ShadowsocksEndpointData, index?: number): ShadowsocksEndpointData;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): BridgeEndpointData.AsObject;
    static toObject(includeInstance: boolean, msg: BridgeEndpointData): BridgeEndpointData.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: BridgeEndpointData, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): BridgeEndpointData;
    static deserializeBinaryFromReader(message: BridgeEndpointData, reader: jspb.BinaryReader): BridgeEndpointData;
}

export namespace BridgeEndpointData {
    export type AsObject = {
        shadowsocksList: Array<ShadowsocksEndpointData.AsObject>,
    }
}

export class ShadowsocksEndpointData extends jspb.Message { 
    getPort(): number;
    setPort(value: number): ShadowsocksEndpointData;
    getCipher(): string;
    setCipher(value: string): ShadowsocksEndpointData;
    getPassword(): string;
    setPassword(value: string): ShadowsocksEndpointData;
    getProtocol(): TransportProtocol;
    setProtocol(value: TransportProtocol): ShadowsocksEndpointData;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ShadowsocksEndpointData.AsObject;
    static toObject(includeInstance: boolean, msg: ShadowsocksEndpointData): ShadowsocksEndpointData.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ShadowsocksEndpointData, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ShadowsocksEndpointData;
    static deserializeBinaryFromReader(message: ShadowsocksEndpointData, reader: jspb.BinaryReader): ShadowsocksEndpointData;
}

export namespace ShadowsocksEndpointData {
    export type AsObject = {
        port: number,
        cipher: string,
        password: string,
        protocol: TransportProtocol,
    }
}

export class WireguardEndpointData extends jspb.Message { 
    clearPortRangesList(): void;
    getPortRangesList(): Array<PortRange>;
    setPortRangesList(value: Array<PortRange>): WireguardEndpointData;
    addPortRanges(value?: PortRange, index?: number): PortRange;
    getIpv4Gateway(): string;
    setIpv4Gateway(value: string): WireguardEndpointData;
    getIpv6Gateway(): string;
    setIpv6Gateway(value: string): WireguardEndpointData;
    clearShadowsocksPortRangesList(): void;
    getShadowsocksPortRangesList(): Array<PortRange>;
    setShadowsocksPortRangesList(value: Array<PortRange>): WireguardEndpointData;
    addShadowsocksPortRanges(value?: PortRange, index?: number): PortRange;
    clearUdp2tcpPortsList(): void;
    getUdp2tcpPortsList(): Array<number>;
    setUdp2tcpPortsList(value: Array<number>): WireguardEndpointData;
    addUdp2tcpPorts(value: number, index?: number): number;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): WireguardEndpointData.AsObject;
    static toObject(includeInstance: boolean, msg: WireguardEndpointData): WireguardEndpointData.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: WireguardEndpointData, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): WireguardEndpointData;
    static deserializeBinaryFromReader(message: WireguardEndpointData, reader: jspb.BinaryReader): WireguardEndpointData;
}

export namespace WireguardEndpointData {
    export type AsObject = {
        portRangesList: Array<PortRange.AsObject>,
        ipv4Gateway: string,
        ipv6Gateway: string,
        shadowsocksPortRangesList: Array<PortRange.AsObject>,
        udp2tcpPortsList: Array<number>,
    }
}

export class PortRange extends jspb.Message { 
    getFirst(): number;
    setFirst(value: number): PortRange;
    getLast(): number;
    setLast(value: number): PortRange;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PortRange.AsObject;
    static toObject(includeInstance: boolean, msg: PortRange): PortRange.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PortRange, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PortRange;
    static deserializeBinaryFromReader(message: PortRange, reader: jspb.BinaryReader): PortRange;
}

export namespace PortRange {
    export type AsObject = {
        first: number,
        last: number,
    }
}

export class AccountAndDevice extends jspb.Message { 
    getAccountNumber(): string;
    setAccountNumber(value: string): AccountAndDevice;

    hasDevice(): boolean;
    clearDevice(): void;
    getDevice(): Device | undefined;
    setDevice(value?: Device): AccountAndDevice;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AccountAndDevice.AsObject;
    static toObject(includeInstance: boolean, msg: AccountAndDevice): AccountAndDevice.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AccountAndDevice, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AccountAndDevice;
    static deserializeBinaryFromReader(message: AccountAndDevice, reader: jspb.BinaryReader): AccountAndDevice;
}

export namespace AccountAndDevice {
    export type AsObject = {
        accountNumber: string,
        device?: Device.AsObject,
    }
}

export class Device extends jspb.Message { 
    getId(): string;
    setId(value: string): Device;
    getName(): string;
    setName(value: string): Device;
    getPubkey(): Uint8Array | string;
    getPubkey_asU8(): Uint8Array;
    getPubkey_asB64(): string;
    setPubkey(value: Uint8Array | string): Device;
    getHijackDns(): boolean;
    setHijackDns(value: boolean): Device;

    hasCreated(): boolean;
    clearCreated(): void;
    getCreated(): google_protobuf_timestamp_pb.Timestamp | undefined;
    setCreated(value?: google_protobuf_timestamp_pb.Timestamp): Device;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Device.AsObject;
    static toObject(includeInstance: boolean, msg: Device): Device.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Device, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Device;
    static deserializeBinaryFromReader(message: Device, reader: jspb.BinaryReader): Device;
}

export namespace Device {
    export type AsObject = {
        id: string,
        name: string,
        pubkey: Uint8Array | string,
        hijackDns: boolean,
        created?: google_protobuf_timestamp_pb.Timestamp.AsObject,
    }
}

export class DeviceList extends jspb.Message { 
    clearDevicesList(): void;
    getDevicesList(): Array<Device>;
    setDevicesList(value: Array<Device>): DeviceList;
    addDevices(value?: Device, index?: number): Device;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeviceList.AsObject;
    static toObject(includeInstance: boolean, msg: DeviceList): DeviceList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeviceList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeviceList;
    static deserializeBinaryFromReader(message: DeviceList, reader: jspb.BinaryReader): DeviceList;
}

export namespace DeviceList {
    export type AsObject = {
        devicesList: Array<Device.AsObject>,
    }
}

export class DeviceRemoval extends jspb.Message { 
    getAccountNumber(): string;
    setAccountNumber(value: string): DeviceRemoval;
    getDeviceId(): string;
    setDeviceId(value: string): DeviceRemoval;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeviceRemoval.AsObject;
    static toObject(includeInstance: boolean, msg: DeviceRemoval): DeviceRemoval.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeviceRemoval, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeviceRemoval;
    static deserializeBinaryFromReader(message: DeviceRemoval, reader: jspb.BinaryReader): DeviceRemoval;
}

export namespace DeviceRemoval {
    export type AsObject = {
        accountNumber: string,
        deviceId: string,
    }
}

export class DeviceState extends jspb.Message { 
    getState(): DeviceState.State;
    setState(value: DeviceState.State): DeviceState;

    hasDevice(): boolean;
    clearDevice(): void;
    getDevice(): AccountAndDevice | undefined;
    setDevice(value?: AccountAndDevice): DeviceState;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeviceState.AsObject;
    static toObject(includeInstance: boolean, msg: DeviceState): DeviceState.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeviceState, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeviceState;
    static deserializeBinaryFromReader(message: DeviceState, reader: jspb.BinaryReader): DeviceState;
}

export namespace DeviceState {
    export type AsObject = {
        state: DeviceState.State,
        device?: AccountAndDevice.AsObject,
    }

    export enum State {
    LOGGED_IN = 0,
    LOGGED_OUT = 1,
    REVOKED = 2,
    }

}

export class DeviceEvent extends jspb.Message { 
    getCause(): DeviceEvent.Cause;
    setCause(value: DeviceEvent.Cause): DeviceEvent;

    hasNewState(): boolean;
    clearNewState(): void;
    getNewState(): DeviceState | undefined;
    setNewState(value?: DeviceState): DeviceEvent;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DeviceEvent.AsObject;
    static toObject(includeInstance: boolean, msg: DeviceEvent): DeviceEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DeviceEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DeviceEvent;
    static deserializeBinaryFromReader(message: DeviceEvent, reader: jspb.BinaryReader): DeviceEvent;
}

export namespace DeviceEvent {
    export type AsObject = {
        cause: DeviceEvent.Cause,
        newState?: DeviceState.AsObject,
    }

    export enum Cause {
    LOGGED_IN = 0,
    LOGGED_OUT = 1,
    REVOKED = 2,
    UPDATED = 3,
    ROTATED_KEY = 4,
    }

}

export class RemoveDeviceEvent extends jspb.Message { 
    getAccountNumber(): string;
    setAccountNumber(value: string): RemoveDeviceEvent;
    clearNewDeviceListList(): void;
    getNewDeviceListList(): Array<Device>;
    setNewDeviceListList(value: Array<Device>): RemoveDeviceEvent;
    addNewDeviceList(value?: Device, index?: number): Device;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RemoveDeviceEvent.AsObject;
    static toObject(includeInstance: boolean, msg: RemoveDeviceEvent): RemoveDeviceEvent.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RemoveDeviceEvent, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RemoveDeviceEvent;
    static deserializeBinaryFromReader(message: RemoveDeviceEvent, reader: jspb.BinaryReader): RemoveDeviceEvent;
}

export namespace RemoveDeviceEvent {
    export type AsObject = {
        accountNumber: string,
        newDeviceListList: Array<Device.AsObject>,
    }
}

export class LeakInfo extends jspb.Message { 
    clearIpAddrsList(): void;
    getIpAddrsList(): Array<string>;
    setIpAddrsList(value: Array<string>): LeakInfo;
    addIpAddrs(value: string, index?: number): string;
    getInterface(): string;
    setInterface(value: string): LeakInfo;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LeakInfo.AsObject;
    static toObject(includeInstance: boolean, msg: LeakInfo): LeakInfo.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LeakInfo, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LeakInfo;
    static deserializeBinaryFromReader(message: LeakInfo, reader: jspb.BinaryReader): LeakInfo;
}

export namespace LeakInfo {
    export type AsObject = {
        ipAddrsList: Array<string>,
        pb_interface: string,
    }
}

export class PlayPurchase extends jspb.Message { 
    getProductId(): string;
    setProductId(value: string): PlayPurchase;

    hasPurchaseToken(): boolean;
    clearPurchaseToken(): void;
    getPurchaseToken(): PlayPurchasePaymentToken | undefined;
    setPurchaseToken(value?: PlayPurchasePaymentToken): PlayPurchase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PlayPurchase.AsObject;
    static toObject(includeInstance: boolean, msg: PlayPurchase): PlayPurchase.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PlayPurchase, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PlayPurchase;
    static deserializeBinaryFromReader(message: PlayPurchase, reader: jspb.BinaryReader): PlayPurchase;
}

export namespace PlayPurchase {
    export type AsObject = {
        productId: string,
        purchaseToken?: PlayPurchasePaymentToken.AsObject,
    }
}

export class PlayPurchasePaymentToken extends jspb.Message { 
    getToken(): string;
    setToken(value: string): PlayPurchasePaymentToken;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): PlayPurchasePaymentToken.AsObject;
    static toObject(includeInstance: boolean, msg: PlayPurchasePaymentToken): PlayPurchasePaymentToken.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: PlayPurchasePaymentToken, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): PlayPurchasePaymentToken;
    static deserializeBinaryFromReader(message: PlayPurchasePaymentToken, reader: jspb.BinaryReader): PlayPurchasePaymentToken;
}

export namespace PlayPurchasePaymentToken {
    export type AsObject = {
        token: string,
    }
}

export class AllowedIpsList extends jspb.Message { 
    clearValuesList(): void;
    getValuesList(): Array<string>;
    setValuesList(value: Array<string>): AllowedIpsList;
    addValues(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AllowedIpsList.AsObject;
    static toObject(includeInstance: boolean, msg: AllowedIpsList): AllowedIpsList.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AllowedIpsList, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AllowedIpsList;
    static deserializeBinaryFromReader(message: AllowedIpsList, reader: jspb.BinaryReader): AllowedIpsList;
}

export namespace AllowedIpsList {
    export type AsObject = {
        valuesList: Array<string>,
    }
}

export class LogFilter extends jspb.Message { 
    getLogFilter(): string;
    setLogFilter(value: string): LogFilter;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LogFilter.AsObject;
    static toObject(includeInstance: boolean, msg: LogFilter): LogFilter.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LogFilter, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LogFilter;
    static deserializeBinaryFromReader(message: LogFilter, reader: jspb.BinaryReader): LogFilter;
}

export namespace LogFilter {
    export type AsObject = {
        logFilter: string,
    }
}

export class LogMessage extends jspb.Message { 
    getMessage(): string;
    setMessage(value: string): LogMessage;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LogMessage.AsObject;
    static toObject(includeInstance: boolean, msg: LogMessage): LogMessage.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LogMessage, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LogMessage;
    static deserializeBinaryFromReader(message: LogMessage, reader: jspb.BinaryReader): LogMessage;
}

export namespace LogMessage {
    export type AsObject = {
        message: string,
    }
}

export enum AfterDisconnect {
    NOTHING = 0,
    BLOCK = 1,
    RECONNECT = 2,
}

export enum FeatureIndicator {
    QUANTUM_RESISTANCE = 0,
    MULTIHOP = 1,
    SPLIT_TUNNELING = 2,
    LOCKDOWN_MODE = 3,
    WIREGUARD_PORT = 4,
    UDP_2_TCP = 5,
    SHADOWSOCKS = 6,
    QUIC = 7,
    LWO = 8,
    LAN_SHARING = 9,
    DNS_CONTENT_BLOCKERS = 10,
    CUSTOM_DNS = 11,
    SERVER_IP_OVERRIDE = 12,
    CUSTOM_MTU = 13,
    DAITA = 14,
    DAITA_MULTIHOP = 15,
}

export enum Ownership {
    ANY = 0,
    MULLVAD_OWNED = 1,
    RENTED = 2,
}

export enum IpVersion {
    V4 = 0,
    V6 = 1,
}

export enum TransportProtocol {
    UDP = 0,
    TCP = 1,
}
