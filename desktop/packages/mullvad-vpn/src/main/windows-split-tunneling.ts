import { app } from 'electron';
import fs from 'fs';
import path from 'path';
import { readShortcut } from 'win-shortcuts';

import {
  ISplitTunnelingApplication,
  ISplitTunnelingAppListRetriever,
} from '../shared/application-types';
import log from '../shared/logging';
import {
  ArrayValue,
  DOS_HEADER,
  DWORD,
  IMAGE_DATA_DIRECTORY,
  IMAGE_DIRECTORY_ENTRY_IMPORT,
  IMAGE_DIRECTORY_ENTRY_RESOURCE,
  IMAGE_FILE_HEADER,
  IMAGE_IMPORT_MODULE_DIRECTORY,
  IMAGE_NT_HEADERS,
  IMAGE_NT_HEADERS64,
  IMAGE_OPTIONAL_HEADER32,
  IMAGE_RESOURCE_DIRECTORY,
  IMAGE_RESOURCE_DIRECTORY_DATA_ENTRY,
  IMAGE_RESOURCE_DIRECTORY_ID_ENTRY,
  ImageNtHeadersUnion,
  ImageOptionalHeaderUnion,
  PrimitiveValue,
  rvaToOffset as rvaToOffsetImpl,
  STRING_FILE_INFO,
  STRING_TABLE,
  STRING_TABLE_STRING,
  StructValue,
  StructWrapper,
  Value,
  VS_VERSIONINFO,
} from './windows-pe-parser';

interface ShortcutDetails {
  target: string;
  name: string;
  args?: string;
  deletable: boolean;
}

type RvaToOffset = (rva: number) => Promise<number>;

// Applications are found by scanning the start menu directories
const APPLICATION_PATHS = [
  `${process.env.ProgramData}/Microsoft/Windows/Start Menu/Programs`,
  `${process.env.AppData}/Microsoft/Windows/Start Menu/Programs`,
];

// Some applications might be falsely filtered from the application list. This allow-list specifies
// apps that are falsely filtered but should be included.
const APPLICATION_ALLOW_LIST = [
  'firefox.exe',
  'chrome.exe',
  'msedge.exe',
  'brave.exe',
  'iexplore.exe',
];

export class WindowsSplitTunnelingAppListRetriever implements ISplitTunnelingAppListRetriever {
  // Cache of all previously scanned shortcuts.
  private shortcutCache: Record<string, ShortcutDetails> = {};
  // Cache of all previously scanned applications.
  private applicationCache: Record<string, ISplitTunnelingApplication> = {};
  // List of shortcuts that have been added manually by the user.
  private additionalShortcuts: ShortcutDetails[] = [];

  // Finds applications by searching through the startmenu for shortcuts with and exe-file as
  // target.
  public async getApplications(
    updateCaches = false,
  ): Promise<{ fromCache: boolean; applications: ISplitTunnelingApplication[] }> {
    const cacheIsEmpty = Object.keys(this.shortcutCache).length === 0;

    const fromCache = !updateCaches && !cacheIsEmpty;
    if (!fromCache) {
      await this.updateShortcutCache();
    }

    await this.updateApplicationCache();

    return {
      fromCache,
      applications: Object.values(this.applicationCache),
    };
  }

  public async getMetadataForApplications(
    applicationPaths: string[],
  ): Promise<{ fromCache: boolean; applications: ISplitTunnelingApplication[] }> {
    // Add excluded apps that are missing from the shortcut cache to it
    await Promise.all(
      applicationPaths.map((applicationPath) =>
        this.addApplicationToAdditionalShortcuts(applicationPath),
      ),
    );

    const applications = await this.getApplications();
    // If applicationPaths is supplied the returnvalue should only contain the applications
    // corresponding to those paths.
    applications.applications = applications.applications.filter(
      (application) =>
        applicationPaths.find(
          (applicationPath) =>
            applicationPath.toLowerCase() === application.absolutepath.toLowerCase(),
        ) !== undefined,
    );

    return applications;
  }

  public resolveExecutablePath(providedPath: string): Promise<string> {
    if (path.extname(providedPath) === '.lnk') {
      const target = this.tryReadShortcut(path.resolve(providedPath));
      if (target) {
        return Promise.resolve(target);
      } else {
        return Promise.reject('Failed to resolve shortcut');
      }
    }

    return Promise.resolve(providedPath);
  }

  // Adds either a shortcut or an executable to the additionalShortcuts list
  public async addApplicationPathToCache(applicationPath: string): Promise<void> {
    const parsedPath = path.parse(applicationPath);
    if (parsedPath.ext === '.lnk') {
      const target = this.tryReadShortcut(path.resolve(applicationPath));
      if (target) {
        this.additionalShortcuts.push({
          target,
          name: path.parse(applicationPath).name,
          deletable: true,
        });
      }
    } else {
      await this.addApplicationToAdditionalShortcuts(applicationPath);
    }
  }

  public removeApplicationFromCache(application: ISplitTunnelingApplication): void {
    this.additionalShortcuts = this.additionalShortcuts.filter(
      (shortcut) => shortcut.target !== application.absolutepath,
    );
    delete this.applicationCache[application.absolutepath.toLowerCase()];
  }

  // Reads the start-menu directories and adds all shortcuts, targeting applications using networking,
  // to the shortcuts cache. Whether or not an application use networking is determined by checking for
  // "WS2_32.dll" in it's imports.
  private async updateShortcutCache(): Promise<void> {
    const links = await Promise.all(
      APPLICATION_PATHS.map((applicationPath) => this.findAllLinks(applicationPath)),
    );
    const resolvedLinks = this.removeDuplicates(this.resolveLinks(links.flat()));

    const shortcuts: ShortcutDetails[] = [];
    for (const shortcut of resolvedLinks) {
      if (
        APPLICATION_ALLOW_LIST.includes(path.basename(shortcut.target.toLowerCase())) ||
        (await this.importsDll(shortcut.target, 'WS2_32.dll'))
      ) {
        shortcuts.push(shortcut);
        this.shortcutCache[shortcut.target.toLowerCase()] = shortcut;
      }
    }
  }

  private async updateApplicationCache(): Promise<void> {
    const shortcuts = Object.values(this.shortcutCache).concat(this.additionalShortcuts);

    await Promise.all(
      shortcuts.map(async (shortcut) => {
        const lowercaseTarget = shortcut.target.toLowerCase();
        if (this.applicationCache[lowercaseTarget] === undefined) {
          this.applicationCache[lowercaseTarget] =
            await this.convertToSplitTunnelingApplication(shortcut);
        }

        return this.applicationCache[lowercaseTarget];
      }),
    );
  }

  // Add excluded apps that are missing from the shortcut cache to it
  private async addApplicationToAdditionalShortcuts(applicationPath: string): Promise<void> {
    if (
      this.shortcutCache[applicationPath.toLowerCase()] === undefined &&
      !this.additionalShortcuts.some(
        (shortcut) => shortcut.target.toLowerCase() === applicationPath.toLowerCase(),
      )
    ) {
      this.additionalShortcuts.push({
        target: applicationPath,
        name: (await this.getProgramName(applicationPath)) ?? path.parse(applicationPath).name,
        deletable: true,
      });
    }
  }

  // Fins all links in a directory.
  private async findAllLinks(path: string): Promise<string[]> {
    if (path.endsWith('.lnk')) {
      return [path];
    } else {
      const stat = await fs.promises.stat(path);
      if (stat.isDirectory()) {
        const contents = await fs.promises.readdir(path);
        const result = await Promise.all(
          contents.map((item) => this.findAllLinks(`${path}/${item}`)),
        );
        return result.flat();
      } else {
        return [];
      }
    }
  }

  private resolveLinks(linkPaths: string[]): ShortcutDetails[] {
    return linkPaths
      .map((link) => {
        const target = this.tryReadShortcut(path.resolve(link));
        if (target) {
          return {
            target,
            name: path.parse(link).name,
          };
        }
        return null;
      })
      .filter(
        (shortcut): shortcut is ShortcutDetails =>
          shortcut !== null &&
          !shortcut.target.endsWith('Mullvad VPN.exe') &&
          shortcut.target.endsWith('.exe') &&
          !shortcut.target.toLowerCase().includes('install') && // Covers "uninstall" as well.
          !shortcut.name.toLowerCase().includes('install'),
      );
  }

  private async getProgramName(exePath: string): Promise<string | undefined> {
    try {
      return await this.getProductName(exePath);
    } catch {
      return undefined;
    }
  }

  // Removes all duplicate shortcuts.
  private removeDuplicates(shortcuts: ShortcutDetails[]): ShortcutDetails[] {
    const unique = shortcuts.reduce(
      (shortcuts, shortcut) => {
        const lowercaseTarget = shortcut.target.toLowerCase();
        if (shortcuts[lowercaseTarget]) {
          if (
            shortcuts[lowercaseTarget].args &&
            shortcuts[lowercaseTarget].args !== '' &&
            (!shortcut.args || shortcut.args === '')
          ) {
            shortcuts[lowercaseTarget] = shortcut;
          }
        } else {
          shortcuts[lowercaseTarget] = shortcut;
        }
        return shortcuts;
      },
      {} as Record<string, ShortcutDetails>,
    );

    return Object.values(unique);
  }

  private async convertToSplitTunnelingApplication(
    shortcut: ShortcutDetails,
  ): Promise<ISplitTunnelingApplication> {
    return {
      absolutepath: shortcut.target,
      name: shortcut.name,
      icon: await this.retrieveIcon(shortcut.target),
      deletable: shortcut.deletable,
    };
  }

  private async retrieveIcon(exe: string) {
    const icon = await app.getFileIcon(exe, { size: 'large' });
    return icon.toDataURL();
  }

  // Checks if the application at the supplied path imports a specific dll.
  private async importsDll(path: string, dllName: string): Promise<boolean> {
    let fileHandle: fs.promises.FileHandle;
    try {
      fileHandle = await fs.promises.open(path, fs.constants.O_RDONLY);
    } catch {
      return false;
    }

    const imports = await this.getExeImports(fileHandle, path);
    await fileHandle.close();
    return imports.map((name) => name.toLowerCase()).includes(dllName.toLowerCase());
  }

  private async getExeImports(fileHandle: fs.promises.FileHandle, path: string): Promise<string[]> {
    try {
      const tableOffsetResult = await this.getTableOffset(fileHandle, IMAGE_DIRECTORY_ENTRY_IMPORT);
      if (tableOffsetResult) {
        const { offset: importTableOffset, rvaToOffset } = tableOffsetResult;
        const moduleNames = await this.getImportModuleNames(
          fileHandle,
          importTableOffset,
          rvaToOffset,
        );
        return moduleNames;
      } else {
        return [];
      }
    } catch (e) {
      log.error(`Failed to read .exe import table for ${path}.`, e);
      return [];
    }
  }

  private async readString(
    fileHandle: fs.promises.FileHandle,
    offset: number,
    encoding: 'ascii' | 'ucs2',
  ): Promise<{ value: string; endOffset: number }> {
    const characterSize = this.getCharacterSize(encoding);
    const buffer = Buffer.alloc(characterSize);
    await fileHandle.read(buffer, 0, characterSize, offset);

    const nextOffset = offset + characterSize;
    if (buffer.every((value) => value === 0)) {
      return { value: '', endOffset: nextOffset };
    } else {
      const { value: nextValue, endOffset } = await this.readString(
        fileHandle,
        nextOffset,
        encoding,
      );
      const value = buffer.toString(encoding) + nextValue;
      return { value, endOffset };
    }
  }

  private getCharacterSize(encoding: 'ascii' | 'ucs2'): number {
    switch (encoding) {
      case 'ascii':
        return 1;
      case 'ucs2':
        return 2;
    }
  }

  // Finds and returns the NT header.
  private async getNtHeader(
    fileHandle: fs.promises.FileHandle,
  ): Promise<StructValue<ImageNtHeadersUnion>> {
    // Check whether or not the file follows the PE format.
    const dosHeader = await Value.fromFile(fileHandle, 0, DOS_HEADER);
    const eMagic = dosHeader.get<PrimitiveValue>('e_magic').value();
    if (eMagic !== 0x5a4d) {
      throw new Error('Not a PE file');
    }

    const ntHeaderOffset = dosHeader.get<PrimitiveValue>('e_lfanew').value();

    // Check if this is a 32- or 64-bit exe-file and return the correct datatype.
    const ntHeader32 = await Value.fromFile(fileHandle, ntHeaderOffset, IMAGE_NT_HEADERS);
    const signature = ntHeader32.get<PrimitiveValue>('Signature').buffer.toString('ascii');
    if (signature !== 'PE\0\0') {
      throw new Error('Not a PE file');
    }

    const magic = ntHeader32
      .get<StructValue<typeof IMAGE_OPTIONAL_HEADER32>>('OptionalHeader')
      .get<PrimitiveValue>('Magic')
      .value();

    // magic is 0x20b for 64-bit executables.
    return magic === 0x20b
      ? Value.fromFile(fileHandle, ntHeaderOffset, IMAGE_NT_HEADERS64)
      : ntHeader32;
  }

  // Reads the import table and returns a list of the imported DLLs.
  private async getImportModuleNames(
    fileHandle: fs.promises.FileHandle,
    importTableOffset: number,
    rvaToOffset: RvaToOffset,
  ): Promise<string[]> {
    const moduleNames: string[] = [];
    const entrySize = Value.sizeOf(IMAGE_IMPORT_MODULE_DIRECTORY);

    // eslint-disable-next-line no-constant-condition
    for (let i = 0; true; i++) {
      const importEntry = await Value.fromFile(
        fileHandle,
        importTableOffset + i * entrySize,
        IMAGE_IMPORT_MODULE_DIRECTORY,
      );
      const nameRva = importEntry.get('ModuleName').value();

      if (nameRva !== 0x0) {
        const offset = await rvaToOffset(nameRva);

        const { value: name } = await this.readString(fileHandle, offset, 'ascii');
        moduleNames.push(name);
      } else {
        return moduleNames;
      }
    }
  }

  private async getProductName(path: string): Promise<string | undefined> {
    let fileHandle: fs.promises.FileHandle;
    try {
      fileHandle = await fs.promises.open(path, fs.constants.O_RDONLY);
    } catch {
      return undefined;
    }

    try {
      const getTableOffsetResult = await this.getTableOffset(
        fileHandle,
        IMAGE_DIRECTORY_ENTRY_RESOURCE,
      );

      if (getTableOffsetResult) {
        const { offset: resourceTableOffset, rvaToOffset } = getTableOffsetResult;
        const leafOffsets = await this.getResourceTreeLeafOffsets(
          fileHandle,
          resourceTableOffset,
          resourceTableOffset,
          rvaToOffset,
          [[16], [1], [0, 1033]],
        );

        const productName = await leafOffsets.reduce(
          async (alreadyFoundValue, leafOffset) => {
            const value = await alreadyFoundValue;
            if (value) {
              return value;
            } else {
              const strings = await this.getVsVersionInfoStrings(fileHandle, leafOffset);
              return strings.get('FileDescription') ?? strings.get('ProductName');
            }
          },
          Promise.resolve() as Promise<string | undefined>,
        );

        return productName;
      } else {
        return undefined;
      }
    } catch {
      return undefined;
    } finally {
      await fileHandle.close();
    }
  }

  private async getTableOffset(
    fileHandle: fs.promises.FileHandle,
    tableIndex: number,
  ): Promise<{ offset: number; rvaToOffset: RvaToOffset } | undefined> {
    const ntHeader = await this.getNtHeader(fileHandle);
    const fileHeader = ntHeader.get<StructValue<typeof IMAGE_FILE_HEADER>>('FileHeader');
    const optionalHeader = ntHeader.get<StructValue<ImageOptionalHeaderUnion>>('OptionalHeader');

    const tableRva = optionalHeader
      .get<ArrayValue<typeof IMAGE_DATA_DIRECTORY>>('DataDirectory')
      .nth(tableIndex)
      .get('VirtualAddress')
      .value();

    if (tableRva === 0x0) {
      return undefined;
    }

    const numberOfSections = fileHeader.get<PrimitiveValue>('NumberOfSections').value();
    const ntHeaderEndOffset =
      ntHeader.offset +
      ntHeader.get<PrimitiveValue<typeof DWORD>>('Signature').size +
      fileHeader.size +
      fileHeader.get<PrimitiveValue>('SizeOfOptionalHeader').value();

    const rvaToOffset = (rva: number) =>
      rvaToOffsetImpl(fileHandle, rva, numberOfSections, ntHeaderEndOffset);

    const tableOffset = await rvaToOffset(tableRva);

    return { offset: tableOffset, rvaToOffset };
  }

  // Searches the resource tree for the supplied paths and returns the leaves at the end of those
  // paths.
  private async getResourceTreeLeafOffsets(
    fileHandle: fs.promises.FileHandle,
    sectionOffset: number,
    tableOffset: number,
    rvaToOffset: (rva: number) => Promise<number>,
    [ids, ...path]: number[][],
  ): Promise<number[]> {
    const table = await Value.fromFile(fileHandle, tableOffset, IMAGE_RESOURCE_DIRECTORY);

    const numberOfNameEntries = table.get('NumberOfNameEntries').value();
    const numberOfIdEntries = table.get('NumberOfIdEntries').value();

    const leaves: number[] = [];

    for (let i = numberOfNameEntries; i < numberOfNameEntries + numberOfIdEntries; i++) {
      const offset =
        tableOffset +
        Value.sizeOf(IMAGE_RESOURCE_DIRECTORY) +
        i * Value.sizeOf(IMAGE_RESOURCE_DIRECTORY_ID_ENTRY);
      const entry = await Value.fromFile(fileHandle, offset, IMAGE_RESOURCE_DIRECTORY_ID_ENTRY);

      const id = entry.get('Id').value();
      if (!ids.includes(id)) {
        continue;
      }

      let offsetToData = entry.get('OffsetToData').value();
      // If the first bit is 1 then the offset points to another node, otherwise it point to a leaf.
      const isLeaf = (offsetToData & 0x80000000) === 0;

      if (isLeaf && path.length === 0) {
        const leafDataOffset = await this.getResourceTreeLeafValueOffset(
          fileHandle,
          sectionOffset + offsetToData,
          rvaToOffset,
        );

        leaves.push(leafDataOffset);
      } else if (!isLeaf) {
        offsetToData &= 0x7fffffff;

        const subTreeLeaves = await this.getResourceTreeLeafOffsets(
          fileHandle,
          sectionOffset,
          sectionOffset + offsetToData,
          rvaToOffset,
          path,
        );

        leaves.push(...subTreeLeaves);
      } else {
        continue;
      }
    }

    return leaves;
  }

  // Finds the Strings structures within the VS_VERSIONINFO structure and returns the contents.
  private async getVsVersionInfoStrings(
    fileHandle: fs.promises.FileHandle,
    offset: number,
  ): Promise<Map<string, string>> {
    try {
      const stringFileInfoOffset = await this.getVsVersionInfoChildrenOffset(fileHandle, offset);

      const stringTableOffset = await this.getChildrenOffset(
        fileHandle,
        stringFileInfoOffset,
        STRING_FILE_INFO,
        (szKey) => szKey === 'StringFileInfo',
      );
      const stringTable = await Value.fromFile(fileHandle, stringTableOffset, STRING_TABLE);
      const stringTableLength = stringTable.get<PrimitiveValue>('wLength').value();

      const stringsOffset = await this.getChildrenOffset(
        fileHandle,
        stringTableOffset,
        STRING_TABLE,
        (szKey) => szKey.substring(4).toLowerCase() === '04b0',
      );

      const strings = await this.parseStrings(
        fileHandle,
        stringsOffset,
        stringTableOffset + stringTableLength,
      );

      return strings;
    } catch {
      return new Map();
    }
  }

  // Loops through the list of strings and returns a map with the contents.
  private async parseStrings(
    fileHandle: fs.promises.FileHandle,
    stringsOffset: number,
    stringTableEnd: number,
  ): Promise<Map<string, string>> {
    const strings = new Map<string, string>();

    let currentStringOffset = stringsOffset;
    while (currentStringOffset < stringTableEnd) {
      const stringValue = await Value.fromFile(
        fileHandle,
        currentStringOffset,
        STRING_TABLE_STRING,
      );
      const structSize = stringValue.get('wLength').value();
      const valueSize = (stringValue.get('wValueLength').value() - 1) * 2;

      const szKeyOffset = currentStringOffset + stringValue.size;
      const { value: szKey, endOffset } = await this.readString(fileHandle, szKeyOffset, 'ucs2');

      const valueOffset = this.alignDword(endOffset);
      // Some programs specify the value size in bytes instead of words resulting in reading double
      // the length. To make sure we don't read beyond the end offset we calculate the max size to
      // read. The last value is the null termination character.
      const calculatedValueMaxSize = structSize - (valueOffset - currentStringOffset) - 2;
      const valueReadSize = Math.min(valueSize, calculatedValueMaxSize);

      const { buffer } = await fileHandle.read(
        Buffer.alloc(valueReadSize),
        0,
        valueReadSize,
        valueOffset,
      );
      const value = buffer.toString('ucs2');

      strings.set(szKey, value);
      currentStringOffset += this.alignDword(stringValue.get<PrimitiveValue>('wLength').value());
    }

    return strings;
  }

  private async getResourceTreeLeafValueOffset(
    fileHandle: fs.promises.FileHandle,
    offset: number,
    rvaToOffset: (rva: number) => Promise<number>,
  ): Promise<number> {
    const leaf = await Value.fromFile(fileHandle, offset, IMAGE_RESOURCE_DIRECTORY_DATA_ENTRY);
    const valueRva = leaf.get<PrimitiveValue>('DataRVA').value();
    const valueOffset = await rvaToOffset(valueRva);

    return valueOffset;
  }

  // Finds the offset to the Children field in the VS_VERSIONINFO structure.
  private async getVsVersionInfoChildrenOffset(fileHandle: fs.promises.FileHandle, offset: number) {
    const valueValueOffset = await this.getChildrenOffset(
      fileHandle,
      offset,
      VS_VERSIONINFO,
      (szKey) => szKey === 'VS_VERSION_INFO',
    );
    const versionInfo = await Value.fromFile(fileHandle, offset, VS_VERSIONINFO);
    const versionInfoValueLength = versionInfo.get<PrimitiveValue>('wValueLength').value();
    const valuePadding2Offset = valueValueOffset + versionInfoValueLength;
    const valueChildrenOffset = this.alignDword(valuePadding2Offset);

    return valueChildrenOffset;
  }

  // Finds the offset to the Children field in any of the STRING_FILE_INFO, STRING_TABLE and
  // STRING_TABLE_STRING structures.
  private async getChildrenOffset(
    fileHandle: fs.promises.FileHandle,
    offset: number,
    datatype: StructWrapper,
    validateSzKey?: (szKey: string) => boolean,
  ) {
    const szKeyOffset = offset + Value.sizeOf(datatype);
    const { value, endOffset } = await this.readString(fileHandle, szKeyOffset, 'ucs2');
    if (validateSzKey && !validateSzKey(value)) {
      throw new Error(`Invalid szKey "${value}"`);
    }

    return this.alignDword(endOffset);
  }

  private alignDword(offset: number): number {
    return Math.ceil(offset / 4) * 4;
  }

  private tryReadShortcut(appPath: string): string | null {
    try {
      return readShortcut(path.resolve(appPath));
    } catch (e) {
      if (e && typeof e === 'object' && 'message' in e) {
        log.error(`Failed to read .lnk shortcut for ${appPath}.`, e.message);
      } else {
        log.error(`Failed to read .lnk shortcut for ${appPath}.`);
      }
      return null;
    }
  }
}
