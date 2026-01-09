import { createContext, useContext } from 'react';

import { IDevice } from '../../../shared/daemon-rpc-types';
import { useBoolean } from '../../lib/utility-hooks';

type DeviceListItemContextType = Omit<DeviceListItemContextProviderProps, 'children'> & {
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

type DeviceListItemContextProviderProps = React.PropsWithChildren<{
  device: IDevice;
}>;

export const DeviceListItemProvider = ({
  children,
  ...props
}: DeviceListItemContextProviderProps) => {
  const [confirmDialogVisible, showConfirmDialog, hideConfirmDialog] = useBoolean(false);
  const [error, setError, resetError] = useBoolean(false);
  const [deleting, setDeleting, resetDeleting] = useBoolean(false);
  return (
    <DeviceListItemContext.Provider
      value={{
        deleting,
        setDeleting,
        resetDeleting,
        confirmDialogVisible,
        showConfirmDialog,
        hideConfirmDialog,
        error,
        setError,
        resetError,
        ...props,
      }}>
      {children}
    </DeviceListItemContext.Provider>
  );
};

export const useDeviceListItemContext = (): DeviceListItemContextType => {
  const context = useContext(DeviceListItemContext);
  if (!context) {
    throw new Error('useDeviceListItemContext must be used within a DeviceListItemProvider');
  }
  return context;
};
