import {
  DOWNLOAD_UPDATE_DOWNLOADING,
  DOWNLOAD_UPDATE_ERROR,
  DOWNLOAD_UPDATE_PROGRESS,
  DOWNLOAD_UPDATE_READY,
  DOWNLOAD_UPDATE_RESET,
  DOWNLOAD_UPDATE_START,
  DOWNLOAD_UPDATE_VERIFY,
  DownloadUpdateActions,
  DownloadUpdateStatus,
} from './actions';

export interface DownloadUpdateState {
  version?: string;
  status: DownloadUpdateStatus;
  progress: number;
  error?: string;
}

const initialState: DownloadUpdateState = {
  version: undefined,
  status: 'idle',
  progress: 0,
};

export function downloadUpdateReducer(
  state: DownloadUpdateState = initialState,
  action: DownloadUpdateActions,
): DownloadUpdateState {
  switch (action.type) {
    case DOWNLOAD_UPDATE_START:
      return {
        ...state,
        version: action.version,
        status: 'starting',
        progress: 0,
        error: undefined,
      };
    case DOWNLOAD_UPDATE_DOWNLOADING:
      return {
        ...state,
        status: 'downloading',
        error: undefined,
      };
    case DOWNLOAD_UPDATE_PROGRESS:
      return {
        ...state,
        progress: action.progress,
      };
    case DOWNLOAD_UPDATE_VERIFY:
      return {
        ...state,
        status: 'verifying',
      };
    case DOWNLOAD_UPDATE_READY:
      return {
        ...state,
        status: 'readyForInstall',
        progress: 100,
      };
    case DOWNLOAD_UPDATE_ERROR:
      return {
        ...state,
        status: 'error',
        error: action.error,
      };
    case DOWNLOAD_UPDATE_RESET:
      return {
        version: undefined,
        status: 'idle',
        progress: 0,
        error: undefined,
      };
    default:
      return state;
  }
}
