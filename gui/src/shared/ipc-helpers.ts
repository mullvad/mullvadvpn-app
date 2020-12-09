import { ipcMain, ipcRenderer, WebContents } from 'electron';
import log from 'electron-log';
import { capitalize } from './string-helpers';

type Handler<T, R> = (callback: (arg: T) => R) => void;
type Sender<T, R> = (arg: T) => R;
type Notifier<T> = (webContents: WebContents, arg: T) => void;
type Listener<T> = (callback: (arg: T) => void) => void;

interface IpcCall<T, R> {
  direction: 'renderer-to-main' | 'main-to-renderer';
  send: (event: string) => Notifier<T> | Sender<T, R>;
  receive: (event: string) => Listener<T> | Handler<T, R>;
}

interface MainToRenderer<T> extends IpcCall<T, never> {
  direction: 'main-to-renderer';
  send: (event: string) => Notifier<T>;
  receive: (event: string) => Listener<T>;
}

interface RendererToMain<T, R> extends IpcCall<T, R> {
  direction: 'renderer-to-main';
  send: (event: string) => Sender<T, R>;
  receive: (event: string) => Handler<T, R>;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyIpcCall = IpcCall<any, any>;
type Schema = Record<string, Record<string, AnyIpcCall>>;

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
type IpcMain<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcMainKey<string & K, S[G][K]>]: IpcMainFn<S[G][K]>;
  };
};

// Transforms the provided schema to the correct type for the renderer event channel.
type IpcRenderer<S extends Schema> = {
  [G in keyof S]: {
    [K in keyof S[G] as IpcRendererKey<string & K, S[G][K]>]: IpcRendererFn<S[G][K]>;
  };
};

// Preforms the transformation of the main event channel in accordance with the above types.
export function createIpcMain<S extends Schema>(ipc: S): IpcMain<S> {
  return createIpc(ipc, (event, key, spec) => {
    const capitalizedKey = capitalize(key);
    const newKey =
      spec.direction === 'main-to-renderer' ? `notify${capitalizedKey}` : `handle${capitalizedKey}`;
    const newValue = spec.direction === 'main-to-renderer' ? spec.send(event) : spec.receive(event);

    return [newKey, newValue];
  });
}

// Preforms the transformation of the renderer event channel in accordance with the above types.
export function createIpcRenderer<S extends Schema>(ipc: S): IpcRenderer<S> {
  return createIpc(ipc, (event, key, spec) => {
    const newKey = spec.direction === 'main-to-renderer' ? `listen${capitalize(key)}` : key;
    const newValue = spec.direction === 'main-to-renderer' ? spec.receive(event) : spec.send(event);

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
    send: (event: string) => (newValue: T) => ipcRenderer.send(event, newValue),
    receive: (event: string) => (handlerFn: (value: T) => void) => {
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
    send: (event: string) => (newValue: T) => ipcRenderer.sendSync(event, newValue),
    receive: (event: string) => (handlerFn: (value: T) => R) => {
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
    receive: (event: string) => (fn: (value: T) => void) => {
      ipcRenderer.on(event, (_event, newState: T) => fn(newState));
    },
  };
}

function notifyRendererImpl<T>(event: string): Notifier<T> {
  return (webContents: WebContents, value: T) => {
    if (webContents.isDestroyed()) {
      log.error(`sender(${event}): webContents is already destroyed!`);
    } else {
      webContents.send(event, value);
    }
  };
}

type RequestResult<T> = { type: 'success'; value: T } | { type: 'error'; message: string };

function invokeImpl<T, R>(event: string): Sender<T, Promise<R>> {
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

function handle<T, R>(event: string): Handler<T, Promise<R>> {
  return (fn: (arg: T) => Promise<R>) => {
    ipcMain.handle(event, async (_ipcEvent, arg: T) => {
      try {
        return { type: 'success', value: await fn(arg) };
      } catch (error) {
        return { type: 'error', message: error.message || '' };
      }
    });
  };
}
