import { IpcMain as EIpcMain, IpcRenderer as EIpcRenderer, WebContents } from 'electron';
import { capitalize } from './string-helpers';
import log from './logging';

type Handler<T, R> = (callback: (arg: T) => R) => void;
type Sender<T, R> = (arg: T) => R;
type Notifier<T> = (webContents: WebContents | undefined, arg: T) => void;
type Listener<T> = (callback: (arg: T) => void) => void;

interface MainToRenderer<T> {
  direction: 'main-to-renderer';
  send: (event: string, ipcMain: EIpcMain) => Notifier<T>;
  receive: (event: string, ipcRenderer: EIpcRenderer, once?: boolean) => Listener<T>;
}

interface RendererToMain<T, R> {
  direction: 'renderer-to-main';
  send: (event: string, ipcRenderer: EIpcRenderer) => Sender<T, R>;
  receive: (event: string, ipcMain: EIpcMain, once?: boolean) => Handler<T, R>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyIpcCall = MainToRenderer<any> | RendererToMain<any, any>;

type Schema = Record<string, Record<string, AnyIpcCall>>;

// Renames all IPC calls, e.g. `callName` to either `notifyCallName` or `handleCallName` depending
// on direction.
type IpcMainKey<N extends string, I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? [`notify${Capitalize<N>}`]
  : [`handle${Capitalize<N>}`, `handle${Capitalize<N>}Once`];

// Selects either the send or receive function depending on direction.
type IpcMainFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? ReturnType<I['send']>
  : ReturnType<I['receive']>;

// Renames all receiving IPC calls, e.g. `callName` to `listenCallName`.
type IpcRendererKey<
  N extends string,
  I extends AnyIpcCall
> = I['direction'] extends 'main-to-renderer'
  ? [`listen${Capitalize<N>}`, `listen${Capitalize<N>}Once`]
  : [N];

// Selects either the send or receive function depending on direction.
type IpcRendererFn<I extends AnyIpcCall> = I['direction'] extends 'main-to-renderer'
  ? ReturnType<I['receive']>
  : ReturnType<I['send']>;

// Transforms the provided schema to the correct type for the main event channel.
type IpcMain<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcMainKey<string & K, S[G][K]>[number]]: IpcMainFn<S[G][K]>;
  };
};

// Transforms the provided schema to the correct type for the renderer event channel.
type IpcRenderer<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcRendererKey<string & K, S[G][K]>[number]]: IpcRendererFn<S[G][K]>;
  };
};

// Preforms the transformation of the main event channel in accordance with the above types.
export function createIpcMain<S extends Schema>(schema: S, ipcMain: EIpcMain): IpcMain<S> {
  return createIpc(schema, (event, key, spec): [
    string,
    Notifier<unknown> | Handler<unknown, unknown>,
  ][] => {
    const capitalizedKey = capitalize(key);

    if (spec.direction === 'main-to-renderer') {
      return [[`notify${capitalizedKey}`, spec.send(event, ipcMain)]];
    } else {
      return [
        [`handle${capitalizedKey}`, spec.receive(event, ipcMain)],
        [`handle${capitalizedKey}Once`, spec.receive(event, ipcMain, true)],
      ];
    }
  });
}

// Preforms the transformation of the renderer event channel in accordance with the above types.
export function createIpcRenderer<S extends Schema>(
  schema: S,
  ipcRenderer: EIpcRenderer,
): IpcRenderer<S> {
  return createIpc(schema, (event, key, spec) => {
    if (spec.direction === 'main-to-renderer') {
      return [
        [`listen${capitalize(key)}`, spec.receive(event, ipcRenderer)],
        [`listen${capitalize(key)}Once`, spec.receive(event, ipcRenderer, true)],
      ];
    } else {
      return [[key, spec.send(event, ipcRenderer)]];
    }
  });
}

function createIpc<S extends Schema, T, R extends IpcMain<S> | IpcRenderer<S>>(
  ipc: S,
  fn: (event: string, key: string, spec: AnyIpcCall) => [newKey: string, newValue: T][],
): R {
  return Object.fromEntries(
    Object.entries(ipc).map(([groupKey, group]) => {
      const newGroup = Object.fromEntries(
        Object.entries(group).flatMap(([key, spec]) => fn(`${groupKey}-${key}`, key, spec)),
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
    receive: (event, ipcMain, once = false) => (handlerFn: (value: T) => void) => {
      const onFnKey = once ? 'once' : 'on';
      ipcMain[onFnKey](event, (_event, newValue: T) => {
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
    receive: (event, ipcMain, once = false) => (handlerFn: (value: T) => R) => {
      const onFnKey = once ? 'once' : 'on';
      ipcMain[onFnKey](event, (ipcEvent, newValue: T) => {
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
    receive: (event, ipcRenderer, once = false) => (fn: (value: T) => void) => {
      const onFnKey = once ? 'once' : 'on';
      ipcRenderer[onFnKey](event, (_event, newState: T) => fn(newState));
    },
  };
}

function notifyRendererImpl<T>(event: string, _ipcMain: EIpcMain): Notifier<T> {
  return (webContents, value) => {
    if (webContents === undefined) {
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

function handle<T, R>(event: string, ipcMain: EIpcMain, once = false): Handler<T, Promise<R>> {
  return (fn: (arg: T) => Promise<R>) => {
    const onFnKey = once ? 'handleOnce' : 'handle';
    ipcMain[onFnKey](event, async (_ipcEvent, arg: T) => {
      try {
        return { type: 'success', value: await fn(arg) };
      } catch (error) {
        return { type: 'error', message: error.message || '' };
      }
    });
  };
}
