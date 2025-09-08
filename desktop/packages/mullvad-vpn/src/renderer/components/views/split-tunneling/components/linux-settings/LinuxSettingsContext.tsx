import React, { useMemo, useState } from 'react';

import { type ILinuxSplitTunnelingApplication } from '../../../../../../shared/application-types';

type LinuxSettingsContextProviderProps = {
  children: React.ReactNode;
};

type LinuxSettingsContext = {
  applications?: ILinuxSplitTunnelingApplication[];
  browseError?: string;
  searchTerm: string;
  setApplications: (value: ILinuxSplitTunnelingApplication[]) => void;
  setBrowseError: (value?: string) => void;
  setSearchTerm: (value: string) => void;
  setSplitTunnelingSupported: (value: boolean) => void;
  splitTunnelingSupported?: boolean;
};

const LinuxSettingsContext = React.createContext<LinuxSettingsContext | undefined>(undefined);

export const useLinuxSettingsContext = (): LinuxSettingsContext => {
  const context = React.useContext(LinuxSettingsContext);
  if (!context) {
    throw new Error('useLinuxSettingsContext must be used within a LinuxSettingsContext');
  }
  return context;
};

export function LinuxSettingsContextProvider({ children }: LinuxSettingsContextProviderProps) {
  const [applications, setApplications] = useState<ILinuxSplitTunnelingApplication[]>();
  const [browseError, setBrowseError] = useState<string>();
  const [searchTerm, setSearchTerm] = useState('');
  const [splitTunnelingSupported, setSplitTunnelingSupported] = useState<boolean | undefined>(
    undefined,
  );

  const value = useMemo(
    () => ({
      applications,
      browseError,
      searchTerm,
      setApplications,
      setBrowseError,
      setSearchTerm,
      setSplitTunnelingSupported,
      splitTunnelingSupported,
    }),
    [
      applications,
      browseError,
      searchTerm,
      setApplications,
      setBrowseError,
      setSearchTerm,
      setSplitTunnelingSupported,
      splitTunnelingSupported,
    ],
  );

  return <LinuxSettingsContext value={value}>{children}</LinuxSettingsContext>;
}
