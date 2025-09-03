import { IApplication } from '../../../../../../shared/application-types';
import { Flex, Spinner } from '../../../../../lib/components';
import List from '../../../../List';
import { StyledSpinnerRow } from './ApplicationListStyles';
import { applicationGetKey } from './utils';

export type ApplicationListProps<T extends IApplication> = {
  applications: T[] | undefined;
  rowRenderer: (application: T) => React.ReactElement;
  'data-testid'?: string;
};

export function ApplicationList<T extends IApplication>({
  applications,
  rowRenderer,
  ...props
}: ApplicationListProps<T>) {
  if (applications == undefined) {
    return (
      <StyledSpinnerRow>
        <Spinner size="big" />
      </StyledSpinnerRow>
    );
  } else {
    const items = applications.slice().sort((a, b) => a.name.localeCompare(b.name));

    return (
      <Flex $flexDirection="column" data-testid={props['data-testid']}>
        <List data-testid={props['data-testid']} items={items} getKey={applicationGetKey}>
          {rowRenderer}
        </List>
      </Flex>
    );
  }
}
