import { promises as fs } from 'fs';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type Primitive = { size: number; reader: (buffer: Buffer) => any };
export type PrimitiveWrapper = { primitive: Primitive };

export type StructItem = { name: string; datatype: Datatype };
export type Struct = Array<StructItem>;
export type StructWrapper = { struct: Struct };

export type ArrayWrapper = { array: Array<Datatype> };

export type Datatype = PrimitiveWrapper | StructWrapper | ArrayWrapper;

// Type that represent the correct value-type for a given Datatype.
type ValueType<T extends Datatype> = T extends PrimitiveWrapper
  ? PrimitiveValue<T>
  : T extends ArrayWrapper
  ? ArrayValue<T>
  : T extends StructWrapper
  ? StructValue<T>
  : never;

// Represents any kind of parseable value within the PE headers. Value is extended by
// PrimitiveValue, ArrayValue and StructValue.
export class Value<T extends Datatype> {
  public constructor(
    protected fileHandle: fs.FileHandle,
    public readonly buffer: Buffer,
    public readonly datatype: T,
    public readonly offset: number,
  ) {}

  public get size(): number {
    return Value.sizeOf(this.datatype);
  }

  public get endOffset(): number {
    return this.offset + this.size;
  }

  public static sizeOf(datatype: Datatype): number {
    if (Value.isPrimitive(datatype)) {
      return datatype.primitive.size;
    } else if (Value.isArray(datatype)) {
      return datatype.array.reduce((sum, current) => sum + Value.sizeOf(current), 0);
    } else if (Value.isStruct(datatype)) {
      return datatype.struct.reduce((sum, current) => sum + Value.sizeOf(current.datatype), 0);
    } else {
      throw new Error('Not possible');
    }
  }

  // Reads a datatype from a file handle and returns the correct subclass of Value.
  public static async fromFile<T extends Datatype>(
    fileHandle: fs.FileHandle,
    offset: number,
    datatype: T,
  ): Promise<ValueType<T>> {
    const buffer = Buffer.alloc(Value.sizeOf(datatype));
    const { bytesRead } = await fileHandle.read(buffer, 0, buffer.length, offset);

    if (bytesRead < buffer.length) {
      throw new Error('Failed to read datatype');
    }

    return Value.createNew(fileHandle, buffer, datatype, offset);
  }

  protected static isPrimitive(datatype: Datatype): datatype is PrimitiveWrapper {
    return 'primitive' in datatype;
  }
  protected static isArray(datatype: Datatype): datatype is ArrayWrapper {
    return 'array' in datatype;
  }
  protected static isStruct(datatype: Datatype): datatype is StructWrapper {
    return 'struct' in datatype;
  }

  protected createNew<T extends Datatype>(
    buffer: Buffer,
    datatype: T,
    offset: number,
  ): ValueType<T> {
    return Value.createNew(this.fileHandle, buffer, datatype, offset);
  }

  private static createNew<T extends Datatype>(
    fileHandle: fs.FileHandle,
    buffer: Buffer,
    datatype: T,
    offset: number,
  ): ValueType<T> {
    if (Value.isPrimitive(datatype)) {
      return new PrimitiveValue(fileHandle, buffer, datatype, offset) as ValueType<T>;
    } else if (Value.isArray(datatype)) {
      return new ArrayValue(fileHandle, buffer, datatype, offset) as ValueType<T>;
    } else if (Value.isStruct(datatype)) {
      return new StructValue(fileHandle, buffer, datatype, offset) as ValueType<T>;
    } else {
      // This will never happen since the value can't be anything else than the above types.
      throw new Error('No matching value type.');
    }
  }
}

// Calculates the byteoffset from a relative virtual address.
export async function rvaToOffset(
  fileHandle: fs.FileHandle,
  rva: number,
  numberOfSections: number,
  firstSectionHeaderOffset: number,
): Promise<number> {
  for (let i = 0; i < numberOfSections; i++) {
    const sectionHeaderOffset = firstSectionHeaderOffset + i * Value.sizeOf(IMAGE_SECTION_HEADER);
    const sectionHeader = await Value.fromFile(
      fileHandle,
      sectionHeaderOffset,
      IMAGE_SECTION_HEADER,
    );
    const sectionRva = sectionHeader.get('VirtualAddress').value<number>();
    const sectionSize = sectionHeader.get('SizeOfRawData').value<number>();

    if (rva >= sectionRva && rva < sectionRva + sectionSize) {
      const pointerToRawData = sectionHeader.get('PointerToRawData').value();
      return pointerToRawData + (rva - sectionRva);
    }
  }

  throw new Error('Failed to map RVA to offset');
}

export class PrimitiveValue<T extends PrimitiveWrapper = PrimitiveWrapper> extends Value<T> {
  // Parses and returns the value.
  public value<U extends ReturnType<T['primitive']['reader']>>(): ReturnType<
    T['primitive']['reader']
  > {
    const result = this.datatype.primitive.reader(this.buffer);
    if (result === undefined) {
      throw new Error('Failed to read value from buffer');
    } else {
      return result as U;
    }
  }
}

export class ArrayValue<T extends ArrayWrapper = ArrayWrapper> extends Value<T> {
  // Parses and returns the value at the specified index.
  public nth<U extends ValueType<T['array'][number]>>(index: number): U {
    const datatype = this.datatype.array[0];
    const itemSize = Value.sizeOf(datatype);
    const offset = index * itemSize;
    const buffer = this.buffer.slice(offset, offset + itemSize);

    return this.createNew(buffer, datatype, offset) as U;
  }
}

export class StructValue<T extends StructWrapper = StructWrapper> extends Value<T> {
  // Parses and returns the value for the specified key.
  public get<
    U extends ValueType<T['struct'][number]['datatype']>,
    V extends StructItem = T['struct'][number]
  >(name: V['name']): U {
    const index = this.datatype.struct.findIndex((entry) => entry.name === name);
    if (index === -1) {
      throw new Error('No such field');
    }

    const datatype = this.datatype.struct[index].datatype;

    const slicedType = { struct: this.datatype.struct.slice(0, index) };
    const offset = Value.sizeOf(slicedType);
    const size = Value.sizeOf(datatype);
    const buffer = this.buffer.slice(offset, offset + size);

    return this.createNew(buffer, datatype, offset) as U;
  }
}

// Datatype specifications
export const ARRAY = <T>(length: number, innerType: T) => ({
  array: Array<T>(length).fill(innerType),
});
export const USHORT = {
  primitive: { size: 2, reader: (buffer: Buffer) => buffer.readUInt16LE(0) },
};
export const LONG = {
  primitive: { size: 4, reader: (buffer: Buffer) => buffer.readUInt32LE(0) },
};
export const ULONGLONG = {
  primitive: {
    size: 8,
    reader: (_buffer: Buffer) => {
      throw new Error('Not implemented');
    },
  },
};
export const WORD = {
  primitive: { size: 2, reader: (buffer: Buffer) => buffer.readUInt16LE(0) },
};
export const DWORD = {
  primitive: { size: 4, reader: (buffer: Buffer) => buffer.readUInt32LE(0) },
};
export const BYTE = {
  primitive: { size: 1, reader: (buffer: Buffer) => buffer.readInt8(0) },
};
export const UTF8_STRING = (length: number) => ({
  primitive: {
    size: length,
    reader: (_buffer: Buffer) => {
      throw new Error('Not implemented');
    },
  },
});

export const IMAGE_FILE_HEADER = {
  struct: [
    { name: 'Machine', datatype: WORD },
    { name: 'NumberOfSections', datatype: WORD },
    { name: 'TimeDateStamp', datatype: DWORD },
    { name: 'PointerToSymbolTable', datatype: DWORD },
    { name: 'NumberOfSymbols', datatype: DWORD },
    { name: 'SizeOfOptionalHeader', datatype: WORD },
    { name: 'Characteristics', datatype: WORD },
  ],
};

export const IMAGE_DATA_DIRECTORY_ENTRY = {
  struct: [
    { name: 'VirtualAddress', datatype: DWORD },
    { name: 'Size', datatype: DWORD },
  ],
};

export const IMAGE_DATA_DIRECTORY = ARRAY(16, IMAGE_DATA_DIRECTORY_ENTRY);

export const IMAGE_OPTIONAL_HEADER32 = {
  struct: [
    { name: 'Magic', datatype: WORD },
    { name: 'MajorLinkerVersion', datatype: BYTE },
    { name: 'MinorLinkerVersion', datatype: BYTE },
    { name: 'SizeOfCode', datatype: DWORD },
    { name: 'SizeOfInitializedData', datatype: DWORD },
    { name: 'SizeOfUninitializedData', datatype: DWORD },
    { name: 'AddressOfEntryPoint', datatype: DWORD },
    { name: 'BaseOfCode', datatype: DWORD },
    { name: 'BaseOfData', datatype: DWORD },
    { name: 'ImageBase', datatype: DWORD },
    { name: 'SectionAlignment', datatype: DWORD },
    { name: 'FileAlignment', datatype: DWORD },
    { name: 'MajorOperatingSystemVersion', datatype: WORD },
    { name: 'MinorOperatingSystemVersion', datatype: WORD },
    { name: 'MajorImageVersion', datatype: WORD },
    { name: 'MinorImageVersion', datatype: WORD },
    { name: 'MajorSubsystemVersion', datatype: WORD },
    { name: 'MinorSubsystemVersion', datatype: WORD },
    { name: 'Win32VersionValue', datatype: DWORD },
    { name: 'SizeOfImage', datatype: DWORD },
    { name: 'SizeOfHeaders', datatype: DWORD },
    { name: 'CheckSum', datatype: DWORD },
    { name: 'Subsystem', datatype: WORD },
    { name: 'DllCharacteristics', datatype: WORD },
    { name: 'SizeOfStackReserve', datatype: DWORD },
    { name: 'SizeOfStackCommit', datatype: DWORD },
    { name: 'SizeOfHeapReserve', datatype: DWORD },
    { name: 'SizeOfHeapCommit', datatype: DWORD },
    { name: 'LoaderFlags', datatype: DWORD },
    { name: 'NumberOfRvaAndSizes', datatype: DWORD },
    { name: 'DataDirectory', datatype: IMAGE_DATA_DIRECTORY },
  ],
};

export const IMAGE_OPTIONAL_HEADER64 = {
  struct: [
    { name: 'Magic', datatype: WORD },
    { name: 'MajorLinkerVersion', datatype: BYTE },
    { name: 'MinorLinkerVersion', datatype: BYTE },
    { name: 'SizeOfCode', datatype: DWORD },
    { name: 'SizeOfInitializedData', datatype: DWORD },
    { name: 'SizeOfUninitializedData', datatype: DWORD },
    { name: 'AddressOfEntryPoint', datatype: DWORD },
    { name: 'BaseOfCode', datatype: DWORD },
    { name: 'ImageBase', datatype: ULONGLONG },
    { name: 'SectionAlignment', datatype: DWORD },
    { name: 'FileAlignment', datatype: DWORD },
    { name: 'MajorOperatingSystemVersion', datatype: WORD },
    { name: 'MinorOperatingSystemVersion', datatype: WORD },
    { name: 'MajorImageVersion', datatype: WORD },
    { name: 'MinorImageVersion', datatype: WORD },
    { name: 'MajorSubsystemVersion', datatype: WORD },
    { name: 'MinorSubsystemVersion', datatype: WORD },
    { name: 'Win32VersionValue', datatype: DWORD },
    { name: 'SizeOfImage', datatype: DWORD },
    { name: 'SizeOfHeaders', datatype: DWORD },
    { name: 'CheckSum', datatype: DWORD },
    { name: 'Subsystem', datatype: WORD },
    { name: 'DllCharacteristics', datatype: WORD },
    { name: 'SizeOfStackReserve', datatype: ULONGLONG },
    { name: 'SizeOfStackCommit', datatype: ULONGLONG },
    { name: 'SizeOfHeapReserve', datatype: ULONGLONG },
    { name: 'SizeOfHeapCommit', datatype: ULONGLONG },
    { name: 'LoaderFlags', datatype: DWORD },
    { name: 'NumberOfRvaAndSizes', datatype: DWORD },
    { name: 'DataDirectory', datatype: IMAGE_DATA_DIRECTORY },
  ],
};

export const IMAGE_NT_HEADERS = {
  struct: [
    { name: 'Signature', datatype: DWORD },
    { name: 'FileHeader', datatype: IMAGE_FILE_HEADER },
    { name: 'OptionalHeader', datatype: IMAGE_OPTIONAL_HEADER32 },
  ],
};

export const IMAGE_NT_HEADERS64 = {
  struct: [
    { name: 'Signature', datatype: DWORD },
    { name: 'FileHeader', datatype: IMAGE_FILE_HEADER },
    { name: 'OptionalHeader', datatype: IMAGE_OPTIONAL_HEADER64 },
  ],
};

export const DOS_HEADER = {
  struct: [
    { name: 'e_magic', datatype: USHORT },
    { name: 'e_cblp', datatype: USHORT },
    { name: 'e_cp', datatype: USHORT },
    { name: 'e_crlc', datatype: USHORT },
    { name: 'e_cparhdr', datatype: USHORT },
    { name: 'e_minalloc', datatype: USHORT },
    { name: 'e_maxalloc', datatype: USHORT },
    { name: 'e_ss', datatype: USHORT },
    { name: 'e_sp', datatype: USHORT },
    { name: 'e_csum', datatype: USHORT },
    { name: 'e_ip', datatype: USHORT },
    { name: 'e_cs', datatype: USHORT },
    { name: 'e_lfarlc', datatype: USHORT },
    { name: 'e_ovno', datatype: USHORT },
    { name: 'e_res', datatype: ARRAY(4, USHORT) },
    { name: 'e_oemid', datatype: USHORT },
    { name: 'e_oeminfo', datatype: USHORT },
    { name: 'e_res2', datatype: ARRAY(10, USHORT) },
    { name: 'e_lfanew', datatype: LONG },
  ],
};

export const IMAGE_SECTION_HEADER = {
  struct: [
    { name: 'Name', datatype: UTF8_STRING(8) },
    { name: 'PhysicalAddressVirtualSizeUnion', datatype: DWORD }, // TODO? Support unions?
    { name: 'VirtualAddress', datatype: DWORD },
    { name: 'SizeOfRawData', datatype: DWORD },
    { name: 'PointerToRawData', datatype: DWORD },
    { name: 'PointerToRelocations', datatype: DWORD },
    { name: 'PointerToLinenumbers', datatype: DWORD },
    { name: 'NumberOfRelocations', datatype: WORD },
    { name: 'NumberOfLinenumbers', datatype: WORD },
    { name: 'Characteristics', datatype: DWORD },
  ],
};

export const IMAGE_IMPORT_MODULE_DIRECTORY = {
  struct: [
    { name: 'ImportLookupTable', datatype: DWORD },
    { name: 'TimeDateStamp', datatype: DWORD },
    { name: 'ForwarderChain', datatype: DWORD },
    { name: 'ModuleName', datatype: DWORD },
    { name: 'ImportAddressTable', datatype: DWORD },
  ],
};

export const IMAGE_DIRECTORY_ENTRY_IMPORT = 1;

export type ImageNtHeadersUnion = typeof IMAGE_NT_HEADERS | typeof IMAGE_NT_HEADERS64;
export type ImageOptionalHeaderUnion =
  | typeof IMAGE_OPTIONAL_HEADER32
  | typeof IMAGE_OPTIONAL_HEADER64;
