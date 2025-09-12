import { createContext, useContext } from 'react';

import { IDevice } from '../../../shared/daemon-rpc-types';

type DeviceListItemContextType = {
  device: IDevice;
  deleting: boolean;
  setDeleting: () => void;
  resetDeleting: () => void;
  confirmDialogVisible: boolean;
  showConfirmDialog: () => void;
  hideConfirmDialog: () => void;
  error: boolean;
  setError: () => void;
  resetError: () => void;
};

const DeviceListItemContext = createContext<DeviceListItemContextType | undefined>(undefined);

type DeviceListItemContextProviderProps = React.PropsWithChildren<DeviceListItemContextType>;

export const DeviceListItemProvider = ({
  children,
  ...props
}: DeviceListItemContextProviderProps) => {
  return <DeviceListItemContext.Provider value={props}>{children}</DeviceListItemContext.Provider>;
};

export const useDeviceListItemContext = (): DeviceListItemContextType => {
  const context = useContext(DeviceListItemContext);
  if (!context) {
    throw new Error('useDeviceListItemContext must be used within a DeviceListItemProvider');
  }
  return context;
};
