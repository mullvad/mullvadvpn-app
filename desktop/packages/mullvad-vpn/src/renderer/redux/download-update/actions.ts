export type DownloadUpdateStatus =
  | 'idle'
  | 'starting'
  | 'downloading'
  | 'verifying'
  | 'readyForInstall'
  | 'error';

export const DOWNLOAD_UPDATE_START = 'DOWNLOAD_UPDATE_START';
export const DOWNLOAD_UPDATE_DOWNLOADING = 'DOWNLOAD_UPDATE_DOWNLOADING';
export const DOWNLOAD_UPDATE_PROGRESS = 'DOWNLOAD_UPDATE_PROGRESS';
export const DOWNLOAD_UPDATE_VERIFY = 'DOWNLOAD_UPDATE_VERIFY';
export const DOWNLOAD_UPDATE_READY = 'DOWNLOAD_UPDATE_READY';
export const DOWNLOAD_UPDATE_ERROR = 'DOWNLOAD_UPDATE_ERROR';
export const DOWNLOAD_UPDATE_RESET = 'DOWNLOAD_UPDATE_RESET';

export interface DownloadUpdateStartAction {
  type: typeof DOWNLOAD_UPDATE_START;
  version: string;
}

export interface DownloadUpdateDownloadingAction {
  type: typeof DOWNLOAD_UPDATE_DOWNLOADING;
}

export interface DownloadUpdateProgressAction {
  type: typeof DOWNLOAD_UPDATE_PROGRESS;
  progress: number;
}

export interface DownloadUpdateVerifyAction {
  type: typeof DOWNLOAD_UPDATE_VERIFY;
}

export interface DownloadUpdateReadyAction {
  type: typeof DOWNLOAD_UPDATE_READY;
}

export interface DownloadUpdateErrorAction {
  type: typeof DOWNLOAD_UPDATE_ERROR;
  error: string;
}

export interface DownloadUpdateResetAction {
  type: typeof DOWNLOAD_UPDATE_RESET;
}

export type DownloadUpdateActions =
  | DownloadUpdateStartAction
  | DownloadUpdateDownloadingAction
  | DownloadUpdateProgressAction
  | DownloadUpdateVerifyAction
  | DownloadUpdateReadyAction
  | DownloadUpdateErrorAction
  | DownloadUpdateResetAction;

export const downloadUpdateStart = (version: string): DownloadUpdateStartAction => ({
  type: DOWNLOAD_UPDATE_START,
  version,
});

export const downloadUpdateDownloading = (): DownloadUpdateDownloadingAction => ({
  type: DOWNLOAD_UPDATE_DOWNLOADING,
});

export const downloadUpdateProgress = (progress: number): DownloadUpdateProgressAction => ({
  type: DOWNLOAD_UPDATE_PROGRESS,
  progress,
});

export const downloadUpdateVerify = (): DownloadUpdateVerifyAction => ({
  type: DOWNLOAD_UPDATE_VERIFY,
});

export const downloadUpdateReady = (): DownloadUpdateReadyAction => ({
  type: DOWNLOAD_UPDATE_READY,
});

export const downloadUpdateError = (error: string): DownloadUpdateErrorAction => ({
  type: DOWNLOAD_UPDATE_ERROR,
  error,
});

export const downloadUpdateReset = (): DownloadUpdateResetAction => ({
  type: DOWNLOAD_UPDATE_RESET,
});

export const downloadUpdateActions = {
  downloadUpdateStart,
  downloadUpdateDownloading,
  downloadUpdateProgress,
  downloadUpdateVerify,
  downloadUpdateReady,
  downloadUpdateError,
  downloadUpdateReset,
};
