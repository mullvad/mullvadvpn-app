import { type ILinuxSplitTunnelingApplication } from '../../../../../../../../../../shared/application-types';

export type LinuxApplicationRowProps = {
  application: ILinuxSplitTunnelingApplication;
  onSelect?: (application: ILinuxSplitTunnelingApplication) => void;
};
