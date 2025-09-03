import React, { useMemo, useState } from 'react';

import { type ISplitTunnelingApplication } from '../../../../../../shared/application-types';

type SettingsContextProviderProps = {
  children: React.ReactNode;
};

type SettingsContext = {
  applications?: ISplitTunnelingApplication[];
  loadingDiskPermissions: boolean;
  searchTerm: string;
  setApplications: (value: ISplitTunnelingApplication[]) => void;
  setLoadingDiskPermissions: (value: boolean) => void;
  setSearchTerm: (value: string) => void;
  setSplitTunnelingAvailable: (value: boolean) => void;
  splitTunnelingAvailable?: boolean;
};

const SettingsContext = React.createContext<SettingsContext | undefined>(undefined);

export const useSettingsContext = (): SettingsContext => {
  const context = React.useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsContext');
  }
  return context;
};

export function SettingsContextProvider({ children }: SettingsContextProviderProps) {
  const [applications, setApplications] = useState<ISplitTunnelingApplication[]>();
  const [loadingDiskPermissions, setLoadingDiskPermissions] = useState(false);
  const [searchTerm, setSearchTerm] = useState('');
  const [splitTunnelingAvailable, setSplitTunnelingAvailable] = useState(
    window.env.platform === 'darwin' ? undefined : true,
  );

  const value = useMemo(
    () => ({
      applications,
      loadingDiskPermissions,
      searchTerm,
      setApplications,
      setLoadingDiskPermissions,
      setSearchTerm,
      setSplitTunnelingAvailable,
      splitTunnelingAvailable,
    }),
    [
      applications,
      loadingDiskPermissions,
      searchTerm,
      setApplications,
      setLoadingDiskPermissions,
      setSearchTerm,
      setSplitTunnelingAvailable,
      splitTunnelingAvailable,
    ],
  );

  return <SettingsContext value={value}>{children}</SettingsContext>;
}
