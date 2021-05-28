import { app, shell } from 'electron';
import fs from 'fs';
import path from 'path';
import { IApplication } from '../shared/application-types';
import log from '../shared/logging';
import {
  ArrayValue,
  DOS_HEADER,
  IMAGE_DATA_DIRECTORY,
  IMAGE_DIRECTORY_ENTRY_IMPORT,
  IMAGE_FILE_HEADER,
  IMAGE_IMPORT_MODULE_DIRECTORY,
  IMAGE_NT_HEADERS,
  IMAGE_NT_HEADERS64,
  ImageNtHeadersUnion,
  IMAGE_OPTIONAL_HEADER32,
  ImageOptionalHeaderUnion,
  PrimitiveValue,
  rvaToOffset,
  StructValue,
  Value,
  DWORD,
} from './windows-pe-parser';

interface ShortcutDetails {
  target: string;
  name: string;
  args?: string;
}

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

// Cache of all previously scanned shortcuts.
const shortcutCache: Record<string, ShortcutDetails> = {};
// Cache of all previously scanned applications.
const applicationCache: Record<string, IApplication> = {};
// List of shortcuts that have been added manually by the user.
const additionalShortcuts: ShortcutDetails[] = [];

// Finds applications by searching through the startmenu for shortcuts with and exe-file as target.
// If applicationPaths has a value, the returned applications are only the ones corresponding to
// those paths.
export async function getApplications(options: {
  applicationPaths?: string[];
  updateCaches?: boolean;
}): Promise<{ fromCache: boolean; applications: IApplication[] }> {
  const cacheIsEmpty = Object.keys(shortcutCache).length === 0;

  if (options.updateCaches || cacheIsEmpty) {
    await updateShortcutCache();
  }

  // Add excluded apps that are missing from the shortcut cache to it
  options.applicationPaths?.forEach(addApplicationToAdditionalShortcuts);

  await updateApplicationCache();
  // If applicationPaths is supplied the returnvalue should only contain the applications
  // corresponding to those paths.
  const applications = Object.values(applicationCache)
    .filter(
      (application) =>
        options.applicationPaths === undefined ||
        options.applicationPaths.includes(application.absolutepath),
    )
    .sort((a, b) => a.name.localeCompare(b.name));

  return {
    fromCache: !options.updateCaches && !cacheIsEmpty,
    applications,
  };
}

// Adds either a shortcut or an executable to the additionalShortcuts list
export function addApplicationPathToCache(applicationPath: string): string {
  const parsedPath = path.parse(applicationPath);
  if (parsedPath.ext === '.lnk') {
    const shortcutDetiails = shell.readShortcutLink(path.resolve(applicationPath));
    additionalShortcuts.push({ ...shortcutDetiails, name: path.parse(applicationPath).name });
    return shortcutDetiails.target;
  } else {
    addApplicationToAdditionalShortcuts(applicationPath);
    return applicationPath;
  }
}

// Reads the start-menu directories and adds all shortcuts, targeting applications using networking,
// to the shortcuts cache. Wheter or not an application use networking is determined by checking for
// "WS2_32.dll" in it's imports.
async function updateShortcutCache(): Promise<void> {
  const links = await Promise.all(APPLICATION_PATHS.map(findAllLinks));
  const resolvedLinks = removeDuplicates(resolveLinks(links.flat()));

  const shortcuts: ShortcutDetails[] = [];
  for (const shortcut of resolvedLinks) {
    if (
      APPLICATION_ALLOW_LIST.includes(path.basename(shortcut.target)) ||
      (await importsDll(shortcut.target, 'WS2_32.dll'))
    ) {
      shortcuts.push(shortcut);
      shortcutCache[shortcut.target] = shortcut;
    }
  }
}

async function updateApplicationCache(): Promise<void> {
  const shortcuts = Object.values(shortcutCache).concat(additionalShortcuts);

  await Promise.all(
    shortcuts.map(async (shortcut) => {
      if (applicationCache[shortcut.target] === undefined) {
        applicationCache[shortcut.target] = await convertToSplitTunnelingApplication(shortcut);
      }

      return applicationCache[shortcut.target];
    }),
  );
}

// Add excluded apps that are missing from the shortcut cache to it
function addApplicationToAdditionalShortcuts(applicationPath: string): void {
  if (
    shortcutCache[applicationPath] === undefined &&
    !additionalShortcuts.some((shortcut) => shortcut.target === applicationPath)
  ) {
    additionalShortcuts.push({
      target: applicationPath,
      name: path.parse(applicationPath).name,
    });
  }
}

// Fins all links in a directory.
async function findAllLinks(path: string): Promise<string[]> {
  if (path.endsWith('.lnk')) {
    return [path];
  } else {
    const stat = await fs.promises.stat(path);
    if (stat.isDirectory()) {
      const contents = await fs.promises.readdir(path);
      const result = await Promise.all(contents.map((item) => findAllLinks(`${path}/${item}`)));
      return result.flat();
    } else {
      return [];
    }
  }
}

function resolveLinks(linkPaths: string[]): ShortcutDetails[] {
  return linkPaths
    .map((link) => {
      try {
        return {
          ...shell.readShortcutLink(path.resolve(link)),
          name: path.parse(link).name,
        };
      } catch (_e) {
        return null;
      }
    })
    .filter(
      (shortcut): shortcut is ShortcutDetails =>
        shortcut !== null &&
        shortcut.name !== 'Mullvad VPN' &&
        shortcut.target.endsWith('.exe') &&
        !shortcut.target.toLowerCase().includes('uninstall') &&
        !shortcut.name.toLowerCase().includes('uninstall'),
    );
}

// Removes all duplicate shortcuts.
function removeDuplicates(shortcuts: ShortcutDetails[]): ShortcutDetails[] {
  const unique = shortcuts.reduce((shortcuts, shortcut) => {
    if (shortcuts[shortcut.target]) {
      if (
        shortcuts[shortcut.target].args &&
        shortcuts[shortcut.target].args !== '' &&
        (!shortcut.args || shortcut.args === '')
      ) {
        shortcuts[shortcut.target] = shortcut;
      }
    } else {
      shortcuts[shortcut.target] = shortcut;
    }
    return shortcuts;
  }, {} as Record<string, ShortcutDetails>);

  return Object.values(unique);
}

async function convertToSplitTunnelingApplication(
  shortcut: ShortcutDetails,
): Promise<IApplication> {
  return {
    absolutepath: shortcut.target,
    name: shortcut.name,
    icon: await retrieveIcon(shortcut.target),
  };
}

async function retrieveIcon(exe: string) {
  const icon = await app.getFileIcon(exe, { size: 'large' });
  return icon.toDataURL();
}

// Checks if the application at the supplied path imports a specific dll.
async function importsDll(path: string, dllName: string): Promise<boolean> {
  let fileHandle: fs.promises.FileHandle;
  try {
    fileHandle = await fs.promises.open(path, fs.constants.O_RDONLY);
  } catch (e) {
    log.error('Failed to create file handle.', e);
    return false;
  }

  const imports = await getExeImports(fileHandle, path);
  await fileHandle.close();
  return imports.map((name) => name.toLowerCase()).includes(dllName.toLowerCase());
}

async function getExeImports(fileHandle: fs.promises.FileHandle, path: string): Promise<string[]> {
  try {
    const ntHeader = await getNtHeader(fileHandle);
    const fileHeader = ntHeader.get<StructValue<typeof IMAGE_FILE_HEADER>>('FileHeader');
    const optionalHeader = ntHeader.get<StructValue<ImageOptionalHeaderUnion>>('OptionalHeader');

    const importTableRva = optionalHeader
      .get<ArrayValue<typeof IMAGE_DATA_DIRECTORY>>('DataDirectory')
      .nth(IMAGE_DIRECTORY_ENTRY_IMPORT)
      .get('VirtualAddress')
      .value();

    if (importTableRva === 0x0) {
      return [];
    }

    const numberOfSections = fileHeader.get<PrimitiveValue>('NumberOfSections').value<number>();
    const ntHeaderEndOffset =
      ntHeader.offset +
      ntHeader.get<PrimitiveValue<typeof DWORD>>('Signature').size +
      fileHeader.size +
      fileHeader.get<PrimitiveValue>('SizeOfOptionalHeader').value();

    const importTableOffset = await rvaToOffset(
      fileHandle,
      importTableRva,
      numberOfSections,
      ntHeaderEndOffset,
    );

    const moduleNames = await getImportModuleNames(
      fileHandle,
      importTableOffset,
      ntHeader.endOffset,
      numberOfSections,
    );

    return moduleNames;
  } catch (e) {
    log.error(`Failed to read .exe import table for ${path}.`, e);
    return [];
  }
}

async function readString(
  fileHandle: fs.promises.FileHandle,
  offset: number,
  buffer = Buffer.alloc(1000),
  index = 0,
): Promise<string> {
  await fileHandle.read(buffer, index, 1, offset + index);
  if (buffer[index] === 0x0) {
    return buffer.slice(0, index).toString('ascii');
  } else {
    return readString(fileHandle, offset, buffer, index + 1);
  }
}

// Finds and returns the NT header.
async function getNtHeader(
  fileHandle: fs.promises.FileHandle,
): Promise<StructValue<ImageNtHeadersUnion>> {
  // Check whether or not the file follows the PE format.
  const dosHeader = await Value.fromFile(fileHandle, 0, DOS_HEADER);
  const eMagic = dosHeader.get<PrimitiveValue>('e_magic').value<number>();
  if (eMagic !== 0x5a4d) {
    throw new Error('Not a PE file');
  }

  const ntHeaderOffset = dosHeader.get<PrimitiveValue>('e_lfanew').value<number>();

  // Check if this is a 32- or 64-bit exe-file and return the correct datatype.
  const ntHeader32 = await Value.fromFile(fileHandle, ntHeaderOffset, IMAGE_NT_HEADERS);
  const signature = ntHeader32.get<PrimitiveValue>('Signature').buffer.toString('ascii');
  if (signature !== 'PE\0\0') {
    throw new Error('Not a PE file');
  }

  const magic = ntHeader32
    .get<StructValue<typeof IMAGE_OPTIONAL_HEADER32>>('OptionalHeader')
    .get<PrimitiveValue>('Magic')
    .value<number>();

  // magic is 0x20b for 64-bit executables.
  return magic === 0x20b
    ? Value.fromFile(fileHandle, ntHeaderOffset, IMAGE_NT_HEADERS64)
    : ntHeader32;
}

// Reads the import table and returns a list of the imported DLLs.
async function getImportModuleNames(
  fileHandle: fs.promises.FileHandle,
  importTableOffset: number,
  firstSectionHeaderOffset: number,
  numberOfSections: number,
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
      const offset = await rvaToOffset(
        fileHandle,
        nameRva,
        numberOfSections,
        firstSectionHeaderOffset,
      );

      const name = await readString(fileHandle, offset);
      moduleNames.push(name);
    } else {
      return moduleNames;
    }
  }
}
