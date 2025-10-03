import { ILinuxSplitTunnelingApplication } from '../../../../src/shared/application-types';

export const linuxApplicationsList: ILinuxSplitTunnelingApplication[] = [
  {
    absolutepath: '/app',
    exec: 'app',
    name: 'app',
    type: 'Application',
    icon: '',
    warning: undefined,
  },
  {
    absolutepath: '/launches-elsewhere',
    exec: 'launches-elsewhere',
    name: 'launches-elsewhere',
    type: 'Application',
    icon: '',
    warning: 'launches-elsewhere',
  },
  {
    absolutepath: '/launches-in-existing-process',
    exec: 'launches-in-existing-process',
    name: 'launches-in-existing-process',
    type: 'Application',
    icon: '',
    warning: 'launches-in-existing-process',
  },
];
