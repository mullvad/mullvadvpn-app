import { ISplitTunnelingApplication } from '../../../../../../shared/application-types';

export type ApplicationRowProps = {
  application: ISplitTunnelingApplication;
  onAdd?: (application: ISplitTunnelingApplication) => void;
  onDelete?: (application: ISplitTunnelingApplication) => void;
  onRemove?: (application: ISplitTunnelingApplication) => void;
};
