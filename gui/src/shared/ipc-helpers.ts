import { IpcMain as EIpcMain, IpcRenderer as EIpcRenderer, WebContents } from 'electron';

import log from './logging';
import { capitalize } from './string-helpers';

type Handler<T, R> = (callback: (arg: T) => R) => void;
type Sender<T, R> = (arg: T) => R;
type Notifier<T> = ((arg: T) => void) | undefined;
type Listener<T> = (callback: (arg: T) => void) => void;

interface MainToRenderer<T> {
  direction: 'main-to-renderer';
  send: (event: string, webContents: WebContents) => Notifier<T>;
  receive: (event: string, ipcRenderer: EIpcRenderer) => Listener<T>;
}

interface RendererToMain<T, R> {
  direction: 'renderer-to-main';
  send: (event: string, ipcRenderer: EIpcRenderer) => Sender<T, R>;
  receive: (event: string, ipcMain: EIpcMain) => Handler<T, R>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyIpcCall = MainToRenderer<any> | RendererToMain<any, any>;

export type Schema = Record<string, Record<string, AnyIpcCall>>;

// Renames all IPC calls, e.g. `callName` to either `notifyCallName` or `handleCallName` depending
// on direction.
type IpcMainKey<N extends string, I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? `notify${Capitalize<N>}`
  : `handle${Capitalize<N>}`;

// Selects either the send or receive function depending on direction.
type IpcMainFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? ReturnType<I['send']>
  : ReturnType<I['receive']>;

// Renames all receiving IPC calls, e.g. `callName` to `listenCallName`.
type IpcRendererKey<
  N extends string,
  I extends AnyIpcCall
> = I['direction'] extends 'main-to-renderer' ? `listen${Capitalize<N>}` : N;

// Selects either the send or receive function depending on direction.
type IpcRendererFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? ReturnType<I['receive']>
  : ReturnType<I['send']>;

// Transforms the provided schema to the correct type for the main event channel.
export type IpcMain<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcMainKey<string & K, S[G][K]>]: IpcMainFn<S[G][K]>;
  };
};

// Transforms the provided schema to the correct type for the renderer event channel.
export type IpcRenderer<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcRendererKey<string & K, S[G][K]>]: IpcRendererFn<S[G][K]>;
  };
};

// Preforms the transformation of the main event channel in accordance with the above types.
export function createIpcMain<S extends Schema>(
  schema: S,
  ipcMain: EIpcMain,
  webContents: WebContents | undefined,
): IpcMain<S> {
  return createIpc(schema, (event, key, spec) => {
    const capitalizedKey = capitalize(key);
    const newKey =
      spec.direction === 'main-to-renderer' ? `notify${capitalizedKey}` : `handle${capitalizedKey}`;

    let newValue;
    if (spec.direction === 'main-to-renderer') {
      newValue = webContents ? spec.send(event, webContents) : undefined;
    } else {
      newValue = spec.receive(event, ipcMain);
    }

    return [newKey, newValue];
  });
}

// Preforms the transformation of the renderer event channel in accordance with the above types.
export function createIpcRenderer<S extends Schema>(
  schema: S,
  ipcRenderer: EIpcRenderer,
): IpcRenderer<S> {
  return createIpc(schema, (event, key, spec) => {
    const newKey = spec.direction === 'main-to-renderer' ? `listen${capitalize(key)}` : key;
    const newValue =
      spec.direction === 'main-to-renderer'
        ? spec.receive(event, ipcRenderer)
        : spec.send(event, ipcRenderer);

    return [newKey, newValue];
  });
}

function createIpc<S extends Schema, T, R extends IpcMain<S> | IpcRenderer<S>>(
  ipc: S,
  fn: (event: string, key: string, spec: AnyIpcCall) => [newKey: string, newValue: T],
): R {
  return Object.fromEntries(
    Object.entries(ipc).map(([groupKey, group]) => {
      const newGroup = Object.fromEntries(
        Object.entries(group).map(([key, spec]) => fn(`${groupKey}-${key}`, key, spec)),
      );
      return [groupKey, newGroup];
    }),
  ) as R;
}

// Sends a request from the renderer process to the main process without any possibility to respond.
export function send<T>(): RendererToMain<T, void> {
  return {
    direction: 'renderer-to-main',
    send: (event, ipcRenderer) => (newValue: T) => ipcRenderer.send(event, newValue),
    receive: (event, ipcMain) => (handlerFn: (value: T) => void) => {
      ipcMain.on(event, (_event, newValue: T) => {
        handlerFn(newValue);
      });
    },
  };
}

// Sends a synchronous request from the renderer process to the main process.
export function invokeSync<T, R>(): RendererToMain<T, R> {
  return {
    direction: 'renderer-to-main',
    send: (event, ipcRenderer) => (newValue: T) => ipcRenderer.sendSync(event, newValue),
    receive: (event, ipcMain) => (handlerFn: (value: T) => R) => {
      ipcMain.on(event, (ipcEvent, newValue: T) => {
        ipcEvent.returnValue = handlerFn(newValue);
      });
    },
  };
}

// Sends an asynchronous request from the renderer process to the main process.
export function invoke<T, R>(): RendererToMain<T, Promise<R>> {
  return {
    direction: 'renderer-to-main',
    send: invokeImpl,
    receive: handle,
  };
}

// Sends a request from the main process to the renderer process without any possibility to respond.
export function notifyRenderer<T>(): MainToRenderer<T> {
  return {
    direction: 'main-to-renderer',
    send: notifyRendererImpl,
    receive: (event, ipcRenderer) => (fn: (value: T) => void) => {
      ipcRenderer.on(event, (_event, newState: T) => fn(newState));
    },
  };
}

function notifyRendererImpl<T>(event: string, webContents: WebContents): Notifier<T> {
  return (value) => {
    if (webContents === undefined || webContents.isDestroyed() || webContents.isCrashed()) {
      log.error(`sender(${event}): webContents is already destroyed!`);
    } else {
      webContents.send(event, value);
    }
  };
}

type RequestResult<T> = { type: 'success'; value: T } | { type: 'error'; message: string };

function invokeImpl<T, R>(event: string, ipcRenderer: EIpcRenderer): Sender<T, Promise<R>> {
  return async (arg: T): Promise<R> => {
    const result: RequestResult<R> = await ipcRenderer.invoke(event, arg);
    switch (result.type) {
      case 'error':
        throw new Error(result.message);
      case 'success':
        return result.value;
    }
  };
}

function handle<T, R>(event: string, ipcMain: EIpcMain): Handler<T, Promise<R>> {
  return (fn: (arg: T) => Promise<R>) => {
    ipcMain.handle(event, async (_ipcEvent, arg: T) => {
      try {
        return { type: 'success', value: await fn(arg) };
      } catch (e) {
        const error = e as Error;
        return { type: 'error', message: error.message || '' };
      }
    });
  };
}
