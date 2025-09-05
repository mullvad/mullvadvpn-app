import React, { useMemo, useState } from 'react';

import { type ISplitTunnelingApplication } from '../../../../../../shared/application-types';

type SplitTunnelingSettingsContextProviderProps = {
  children: React.ReactNode;
};

type SplitTunnelingSettingsContext = {
  applications?: ISplitTunnelingApplication[];
  loadingDiskPermissions: boolean;
  searchTerm: string;
  setApplications: (value: ISplitTunnelingApplication[]) => void;
  setLoadingDiskPermissions: (value: boolean) => void;
  setSearchTerm: (value: string) => void;
  setSplitTunnelingAvailable: (value: boolean) => void;
  splitTunnelingAvailable?: boolean;
};

const SplitTunnelingSettingsContext = React.createContext<
  SplitTunnelingSettingsContext | undefined
>(undefined);

export const useSplitTunnelingSettingsContext = (): SplitTunnelingSettingsContext => {
  const context = React.useContext(SplitTunnelingSettingsContext);
  if (!context) {
    throw new Error(
      'useSplitTunnelingSettingsContext must be used within a SplitTunnelingSettingsContext',
    );
  }
  return context;
};

export function SplitTunnelingSettingsContextProvider({
  children,
}: SplitTunnelingSettingsContextProviderProps) {
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

  return <SplitTunnelingSettingsContext value={value}>{children}</SplitTunnelingSettingsContext>;
}
