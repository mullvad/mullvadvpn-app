import { type IApplication } from '../../../../shared/application-types';

export function includesSearchTerm(application: IApplication, searchTerm: string) {
  return application.name.toLowerCase().includes(searchTerm.toLowerCase());
}

export interface DisabledApplicationProps {
  $lookDisabled?: boolean;
}

export const disabledApplication = (props: DisabledApplicationProps) => ({
  opacity: props.$lookDisabled ? 0.6 : undefined,
});
