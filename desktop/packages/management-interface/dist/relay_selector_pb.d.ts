// package: mullvad_daemon.relay_selector
// file: relay_selector.proto

/* tslint:disable */
/* eslint-disable */

import * as jspb from "google-protobuf";
import * as management_interface_pb from "./management_interface_pb";

export class Predicate extends jspb.Message { 

    hasSinglehop(): boolean;
    clearSinglehop(): void;
    getSinglehop(): EntryConstraints | undefined;
    setSinglehop(value?: EntryConstraints): Predicate;

    hasAutohop(): boolean;
    clearAutohop(): void;
    getAutohop(): EntryConstraints | undefined;
    setAutohop(value?: EntryConstraints): Predicate;

    hasEntry(): boolean;
    clearEntry(): void;
    getEntry(): MultiHopConstraints | undefined;
    setEntry(value?: MultiHopConstraints): Predicate;

    hasExit(): boolean;
    clearExit(): void;
    getExit(): MultiHopConstraints | undefined;
    setExit(value?: MultiHopConstraints): Predicate;

    getContextCase(): Predicate.ContextCase;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Predicate.AsObject;
    static toObject(includeInstance: boolean, msg: Predicate): Predicate.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Predicate, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Predicate;
    static deserializeBinaryFromReader(message: Predicate, reader: jspb.BinaryReader): Predicate;
}

export namespace Predicate {
    export type AsObject = {
        singlehop?: EntryConstraints.AsObject,
        autohop?: EntryConstraints.AsObject,
        entry?: MultiHopConstraints.AsObject,
        exit?: MultiHopConstraints.AsObject,
    }

    export enum ContextCase {
        CONTEXT_NOT_SET = 0,
        SINGLEHOP = 1,
        AUTOHOP = 2,
        ENTRY = 3,
        EXIT = 4,
    }

}

export class EntryConstraints extends jspb.Message { 

    hasGeneralConstraints(): boolean;
    clearGeneralConstraints(): void;
    getGeneralConstraints(): ExitConstraints | undefined;
    setGeneralConstraints(value?: ExitConstraints): EntryConstraints;

    hasObfuscationSettings(): boolean;
    clearObfuscationSettings(): void;
    getObfuscationSettings(): management_interface_pb.ObfuscationSettings | undefined;
    setObfuscationSettings(value?: management_interface_pb.ObfuscationSettings): EntryConstraints;

    hasDaitaSettings(): boolean;
    clearDaitaSettings(): void;
    getDaitaSettings(): management_interface_pb.DaitaSettings | undefined;
    setDaitaSettings(value?: management_interface_pb.DaitaSettings): EntryConstraints;
    getIpVersion(): management_interface_pb.IpVersion;
    setIpVersion(value: management_interface_pb.IpVersion): EntryConstraints;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): EntryConstraints.AsObject;
    static toObject(includeInstance: boolean, msg: EntryConstraints): EntryConstraints.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: EntryConstraints, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): EntryConstraints;
    static deserializeBinaryFromReader(message: EntryConstraints, reader: jspb.BinaryReader): EntryConstraints;
}

export namespace EntryConstraints {
    export type AsObject = {
        generalConstraints?: ExitConstraints.AsObject,
        obfuscationSettings?: management_interface_pb.ObfuscationSettings.AsObject,
        daitaSettings?: management_interface_pb.DaitaSettings.AsObject,
        ipVersion: management_interface_pb.IpVersion,
    }
}

export class ExitConstraints extends jspb.Message { 

    hasLocation(): boolean;
    clearLocation(): void;
    getLocation(): management_interface_pb.LocationConstraint | undefined;
    setLocation(value?: management_interface_pb.LocationConstraint): ExitConstraints;
    clearProvidersList(): void;
    getProvidersList(): Array<Provider>;
    setProvidersList(value: Array<Provider>): ExitConstraints;
    addProviders(value?: Provider, index?: number): Provider;
    getOwnership(): management_interface_pb.Ownership;
    setOwnership(value: management_interface_pb.Ownership): ExitConstraints;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ExitConstraints.AsObject;
    static toObject(includeInstance: boolean, msg: ExitConstraints): ExitConstraints.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ExitConstraints, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ExitConstraints;
    static deserializeBinaryFromReader(message: ExitConstraints, reader: jspb.BinaryReader): ExitConstraints;
}

export namespace ExitConstraints {
    export type AsObject = {
        location?: management_interface_pb.LocationConstraint.AsObject,
        providersList: Array<Provider.AsObject>,
        ownership: management_interface_pb.Ownership,
    }
}

export class MultiHopConstraints extends jspb.Message { 

    hasEntry(): boolean;
    clearEntry(): void;
    getEntry(): EntryConstraints | undefined;
    setEntry(value?: EntryConstraints): MultiHopConstraints;

    hasExit(): boolean;
    clearExit(): void;
    getExit(): ExitConstraints | undefined;
    setExit(value?: ExitConstraints): MultiHopConstraints;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): MultiHopConstraints.AsObject;
    static toObject(includeInstance: boolean, msg: MultiHopConstraints): MultiHopConstraints.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: MultiHopConstraints, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): MultiHopConstraints;
    static deserializeBinaryFromReader(message: MultiHopConstraints, reader: jspb.BinaryReader): MultiHopConstraints;
}

export namespace MultiHopConstraints {
    export type AsObject = {
        entry?: EntryConstraints.AsObject,
        exit?: ExitConstraints.AsObject,
    }
}

export class RelayPartitions extends jspb.Message { 
    clearMatchesList(): void;
    getMatchesList(): Array<Relay>;
    setMatchesList(value: Array<Relay>): RelayPartitions;
    addMatches(value?: Relay, index?: number): Relay;
    clearDiscardsList(): void;
    getDiscardsList(): Array<DiscardedRelay>;
    setDiscardsList(value: Array<DiscardedRelay>): RelayPartitions;
    addDiscards(value?: DiscardedRelay, index?: number): DiscardedRelay;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): RelayPartitions.AsObject;
    static toObject(includeInstance: boolean, msg: RelayPartitions): RelayPartitions.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: RelayPartitions, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): RelayPartitions;
    static deserializeBinaryFromReader(message: RelayPartitions, reader: jspb.BinaryReader): RelayPartitions;
}

export namespace RelayPartitions {
    export type AsObject = {
        matchesList: Array<Relay.AsObject>,
        discardsList: Array<DiscardedRelay.AsObject>,
    }
}

export class Relay extends jspb.Message { 
    getHostname(): string;
    setHostname(value: string): Relay;

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
    }
}

export class DiscardedRelay extends jspb.Message { 

    hasRelay(): boolean;
    clearRelay(): void;
    getRelay(): Relay | undefined;
    setRelay(value?: Relay): DiscardedRelay;

    hasWhy(): boolean;
    clearWhy(): void;
    getWhy(): IncompatibleConstraints | undefined;
    setWhy(value?: IncompatibleConstraints): DiscardedRelay;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): DiscardedRelay.AsObject;
    static toObject(includeInstance: boolean, msg: DiscardedRelay): DiscardedRelay.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: DiscardedRelay, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): DiscardedRelay;
    static deserializeBinaryFromReader(message: DiscardedRelay, reader: jspb.BinaryReader): DiscardedRelay;
}

export namespace DiscardedRelay {
    export type AsObject = {
        relay?: Relay.AsObject,
        why?: IncompatibleConstraints.AsObject,
    }
}

export class IncompatibleConstraints extends jspb.Message { 
    getInactive(): boolean;
    setInactive(value: boolean): IncompatibleConstraints;
    getLocation(): boolean;
    setLocation(value: boolean): IncompatibleConstraints;
    getProviders(): boolean;
    setProviders(value: boolean): IncompatibleConstraints;
    getOwnership(): boolean;
    setOwnership(value: boolean): IncompatibleConstraints;
    getIpVersion(): boolean;
    setIpVersion(value: boolean): IncompatibleConstraints;
    getDaita(): boolean;
    setDaita(value: boolean): IncompatibleConstraints;
    getObfuscation(): boolean;
    setObfuscation(value: boolean): IncompatibleConstraints;
    getPort(): boolean;
    setPort(value: boolean): IncompatibleConstraints;
    getConflictWithOtherHop(): boolean;
    setConflictWithOtherHop(value: boolean): IncompatibleConstraints;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): IncompatibleConstraints.AsObject;
    static toObject(includeInstance: boolean, msg: IncompatibleConstraints): IncompatibleConstraints.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: IncompatibleConstraints, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): IncompatibleConstraints;
    static deserializeBinaryFromReader(message: IncompatibleConstraints, reader: jspb.BinaryReader): IncompatibleConstraints;
}

export namespace IncompatibleConstraints {
    export type AsObject = {
        inactive: boolean,
        location: boolean,
        providers: boolean,
        ownership: boolean,
        ipVersion: boolean,
        daita: boolean,
        obfuscation: boolean,
        port: boolean,
        conflictWithOtherHop: boolean,
    }
}

export class Provider extends jspb.Message { 
    getName(): string;
    setName(value: string): Provider;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Provider.AsObject;
    static toObject(includeInstance: boolean, msg: Provider): Provider.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Provider, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Provider;
    static deserializeBinaryFromReader(message: Provider, reader: jspb.BinaryReader): Provider;
}

export namespace Provider {
    export type AsObject = {
        name: string,
    }
}
