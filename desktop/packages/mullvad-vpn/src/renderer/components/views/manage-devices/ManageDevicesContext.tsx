import React from 'react';

type ManageDevicesContextProps = {
  isFetching: boolean;
  isLoading: boolean;
  refetchDevices: () => Promise<void>;
};

const ManageDevicesContext = React.createContext<ManageDevicesContextProps | undefined>(undefined);

export const useManageDevicesContext = (): ManageDevicesContextProps => {
  const context = React.useContext(ManageDevicesContext);
  if (!context) {
    throw new Error('useManageDevicesContext must be used within a ManageDevicesProvider');
  }
  return context;
};

interface ManageDevicesProviderProps {
  isFetching: boolean;
  isLoading: boolean;
  refetchDevices: () => Promise<void>;
  children: React.ReactNode;
}

export function ManageDevicesProvider({
  isFetching,
  isLoading,
  refetchDevices,
  children,
}: ManageDevicesProviderProps) {
  return (
    <ManageDevicesContext.Provider value={{ isFetching, isLoading, refetchDevices }}>
      {children}
    </ManageDevicesContext.Provider>
  );
}
